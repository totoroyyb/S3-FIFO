use std::hash::Hash;
use super::fifo_cache::FIFOCache;

pub struct S3FIFO<K, V> {
    cache_size: usize,
    small_cache_capacity_ratio: f64,
    small_cache_capacity: usize,
    main_cache_capacity: usize,
    ghost_cache_capacity: usize,

    s_queue: FIFOCache<K, V>,
    m_queue: FIFOCache<K, V>,
    g_queue: FIFOCache<K, V>,

    size: usize
}

impl<K, V> S3FIFO<K,V> 
where 
    K: Default + Clone + Eq + Hash,
    V: Default + Clone,
{
    pub fn new(cache_size: usize, small_cache_ratio: f64) -> S3FIFO<K, V> {
        assert!(small_cache_ratio > 0.0 && small_cache_ratio < 1.0);

        let small_cache_capacity = ((cache_size as f64) * small_cache_ratio) as usize;
        let main_cache_capacity = cache_size - small_cache_capacity;

        assert!(main_cache_capacity > 0);

        // Use the same capacity for ghost and main cache for now.
        let ghost_cache_capacity = main_cache_capacity;

        S3FIFO { 
            cache_size,
            small_cache_capacity_ratio: small_cache_ratio,
            small_cache_capacity, 
            main_cache_capacity, 
            ghost_cache_capacity,
            s_queue: FIFOCache::new(small_cache_capacity), 
            m_queue: FIFOCache::new(main_cache_capacity), 
            g_queue: FIFOCache::new(ghost_cache_capacity),
            size: 0
        }
    }

    pub fn new_with_default_ratio(cache_size: usize) {
        Self::new(cache_size, 0.1);
    }

}

///
/// User-facing/client-facing APIs.
impl<K, V> S3FIFO<K,V> 
where 
    K: Eq + Hash,
{
    pub fn get(&mut self, key: &K) -> Option<&V> 
    where K: Clone 
    {
        if let Some(obj) = self.s_queue.find(key) {
            return Some(&obj);
        }

        if let Some(obj) = self.m_queue.find(key) {
            return Some(&obj);
        }

        return None
    }

    pub fn get_copy(&mut self, key: &K) -> Option<V>
    where K: Clone, V: Clone 
    {
        if let Some(value) = self.get(key) {
            Some(value.clone())
        } else {
            None
        }
    }

    // TODO: TTL supports
    pub fn put(&mut self, key: K, value: V)
    where K: Clone, V: Clone
    {
        if let Some(obj) = self.s_queue.find_mut(&key) {
            obj.set_value(value);
            return;
        } 
        
        if let Some(obj) = self.m_queue.find_mut(&key) {
            obj.set_value(value);
            return;
        }

        // NOT FOUND in cache
        self.insert(key, value);
    }

    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.size == self.cache_size
    }
}

/// 
/// Some internal functions
/// They are develoepr-facing APIs.
impl<K, V> S3FIFO<K, V> 
where 
    K: Clone + Eq + Hash, 
    V: Clone
{
    fn insert(&mut self, key: K, value: V) 
    {
        while self.is_full() { self.evict() }

        // Found in ghost queue
        if let Some(_) = self.g_queue.find(&key) {
            self.m_queue.insert(key, value);
        } else {
            self.s_queue.insert(key, value);
        }

        self.size += 1;
    }

    #[inline(always)]
    fn evict(&mut self) {
        if self.s_queue.is_full() {
            self.evict_s();
        }

        if self.m_queue.is_full() {
            self.evict_m();
        }
    }

    #[inline(always)]
    fn evict_s(&mut self) 
    {
        let mut evicted = false;
        while !evicted && !self.s_queue.empty() {
            if let Some((key, obj)) = self.s_queue.evict() {
                if obj.get_freq() > 1 {
                    self.m_queue.insert(key, obj.get_value_copy());
                    if self.m_queue.is_full() { self.evict_m() }
                } else {
                    self.g_queue.insert(key, obj.get_value_copy());
                    evicted = true;
                }
            }
        }
    }

    #[inline(always)]
    fn evict_m(&mut self) 
    {
        let mut evicted = false;
        while !evicted && !self.m_queue.empty() {
            if let Some((key, obj)) = self.m_queue.evict() {
                if obj.get_freq() > 0 {
                    let mut meta = obj.get_meta_copy(); 
                    meta.desc_freq();
                    
                    self.m_queue.insert_with_meta(
                        key, 
                        obj.get_value_copy(),
                        meta 
                    );
                    // self.m_queue.insert(key, obj.get_value_copy());
                } else {
                    evicted = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::S3FIFO;

    #[test]
    fn init() {
        let cache: S3FIFO<isize, isize> = S3FIFO::new(100, 0.1);

        assert_eq!(cache.cache_size, 100);
        assert_eq!(cache.small_cache_capacity_ratio, 0.1);
        assert_eq!(cache.small_cache_capacity, 10);
        assert_eq!(cache.size, 0);
    }

    #[test]
    fn empty_get() {
        let mut cache: S3FIFO<isize, isize> = S3FIFO::new(100, 0.1);
        let result = cache.get(&1);
        assert!(result.is_none());
        assert_eq!(cache.size, 0);
    }

    #[test]
    fn put_value() {
        let mut cache: S3FIFO<isize, isize> = S3FIFO::new(100, 0.1);
        cache.put(0, 0);

        assert_eq!(cache.size, 1);
        assert_eq!(cache.s_queue.len(), 1);
        assert_eq!(cache.m_queue.len(), 0);
        assert_eq!(cache.g_queue.len(), 0);

        cache.put(1, 1);
        assert_eq!(cache.size, 2);
        assert_eq!(cache.s_queue.len(), 2);
        assert_eq!(cache.m_queue.len(), 0);
        assert_eq!(cache.g_queue.len(), 0);
    }

    #[test]
    fn dup_put() {
        let mut cache: S3FIFO<isize, isize> = S3FIFO::new(100, 0.1);
        cache.put(0, 0);

        assert_eq!(cache.size, 1);
        assert_eq!(cache.s_queue.len(), 1);
        assert_eq!(cache.m_queue.len(), 0);
        assert_eq!(cache.g_queue.len(), 0);

        cache.put(0, 1);
        assert_eq!(cache.size, 1);
        assert_eq!(cache.s_queue.len(), 1);
        assert_eq!(cache.m_queue.len(), 0);
        assert_eq!(cache.g_queue.len(), 0);
    }

    #[test]
    fn simple_put_get() {
        let mut cache: S3FIFO<isize, isize> = S3FIFO::new(100, 0.1);
        cache.put(0, 0);
        let result = cache.get(&0);
        assert_eq!(result, Some(&0));
    }

    #[test]
    fn multiple_put_get() {
        let mut cache: S3FIFO<isize, isize> = S3FIFO::new(100, 0.1);
        for i in 0..50 {
            cache.put(i, i);
            let result = cache.get(&i);
            assert_eq!(result, Some(&i));
        }
    }
}
