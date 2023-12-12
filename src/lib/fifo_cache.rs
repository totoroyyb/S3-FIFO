use std::cmp::min;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;

use super::ring_buffer::RingBuffer;

#[derive(Default, Clone)]
pub struct CacheMetadata {
    freq: usize,
}

impl CacheMetadata {
    #[inline(always)]
    pub fn inc_freq(&mut self) {
        self.freq = min(self.freq + 1, 3);
    }

    #[inline(always)]
    pub fn desc_freq(&mut self) {
        if self.freq != 0 { self.freq -= 1; }
    }
}

#[derive(Default, Clone)]
pub struct CacheObject<V> {
    value: V,
    meta: CacheMetadata
}

impl<V> CacheObject<V> {
    #[inline(always)]
    fn inc_freq(&mut self) {
        self.meta.inc_freq();
    }

    #[inline(always)]
    fn desc_freq(&mut self) {
        self.meta.desc_freq();
    }

    #[inline(always)]
    pub fn set_value(&mut self, value: V) {
        self.value = value;
    }

    #[inline(always)]
    pub fn get_value(&self) -> &V {
        &self.value
    }

    #[inline(always)]
    pub fn get_value_copy(&self) -> V where V: Clone {
        self.value.clone()
    }

    #[inline(always)]
    pub fn get_freq(&self) -> usize {
        self.meta.freq
    }

    #[inline(always)]
    pub fn get_meta(&self) -> &CacheMetadata {
        &self.meta
    }

    #[inline(always)]
    pub fn get_meta_copy(&self) -> CacheMetadata {
        self.meta.clone()
    }
}

impl<V> Deref for CacheObject<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.get_value()
    }
}

pub struct FIFOCache<K, V> {
    rb: RingBuffer<K>,
    hashtable: HashMap<K, CacheObject<V>>,
}

impl<K, V> FIFOCache<K, V>
where 
    K: Default + Clone + Eq + Hash, 
    V: Default + Clone 
{
    #[inline]
    #[must_use]
    pub fn new(capacity: usize) -> FIFOCache<K, V> {
        FIFOCache { 
            rb: RingBuffer::new(capacity), 
            hashtable: HashMap::new()
        }
    }

    ///
    /// No-op if the key isn't present.
    /// 
    /// **DO NOT USE THIS FUNCTION FOR NOW**
    pub fn update(&mut self, key: K, value: V) {
        if self.hashtable.contains_key(&key) {
            self.hashtable
                .entry(key)
                .and_modify(|obj| { 
                    obj.set_value(value);
                    // Should we manage metadat in this level???
                    // obj.inc_freq()
                });
        }
    }
}

impl<K, V> FIFOCache<K, V>
where 
    K: Clone + Eq + Hash, 
{
    ///
    /// Safety: 
    /// insert will potentially overwrite elements in the RingBuffer 
    /// if the number of elements exceeds the capacity.
    pub fn insert(&mut self, key: K, value: V) {
        let meta = CacheMetadata::default();
        self.insert_with_meta(key, value, meta);
    }

    pub fn insert_with_meta(&mut self, key: K, value: V, meta: CacheMetadata) {
        self.hashtable.insert(
            key.clone(), 
            CacheObject { value, meta }
        );
        self.rb.push_back(key);
 
    }

    pub fn evict(&mut self) -> Option<(K, CacheObject<V>)> {
        let key = self.rb.pop_front();
        if let Some(key) = key {
            self.hashtable.remove_entry(&key)
        } else {
            None
        }
    }
}

impl<K, V> FIFOCache<K, V>
{
    #[inline(always)]
    fn inc_freq(&mut self, key: &K) 
    where K: Eq + Hash + Clone
    {
        self.hashtable.entry(key.clone()).and_modify(|obj| {
            obj.inc_freq();
        });
    }


    // Separate impl block more generic trait bound
    #[inline(always)]
    pub fn find(&mut self, key: &K) -> Option<&CacheObject<V>>
    where K: Eq + Hash + Clone
    {
        self.inc_freq(key);
        self.hashtable.get(key)
    }

    #[inline(always)]
    pub fn find_mut(&mut self, key: &K) -> Option<&mut CacheObject<V>> 
    where K: Eq + Hash + Clone
    {
        self.inc_freq(key);
        self.hashtable.get_mut(key)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.rb.len()
    }

    #[inline(always)]
    pub fn empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> FIFOCache<K, V> {
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.rb.is_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut cache: FIFOCache<isize, isize> = FIFOCache::new(5);
        cache.insert(0, 0);
        cache.insert(1, 1);

        let rb_results = cache.rb.get_values();
        assert_eq!(rb_results, vec![0, 1]);
        
        assert_eq!(cache.hashtable.len(), 2);

        let cache_obj = cache.hashtable.get(&0);
        if let Some(cache_obj) = cache_obj {
            assert_eq!(cache_obj.deref(), &0);
        } else {
            panic!("cache_obj should exist");
        }

        let cache_obj = cache.hashtable.get(&1);
        if let Some(cache_obj) = cache_obj {
            assert_eq!(cache_obj.deref(), &1);
        } else {
            panic!("cache_obj should exist");
        }
    }

    #[test]
    fn find() {
        let mut cache: FIFOCache<isize, isize> = FIFOCache::new(5);
        cache.insert(0, 0);
        cache.insert(1, 1);
        let value = cache.find(&0).unwrap().deref();
        assert_eq!(value, &0);

        let value = cache.find(&1).unwrap().deref();
        assert_eq!(value, &1);

        let result = cache.find(&2);
        assert!(result.is_none());
    }

    #[test]
    fn full_cache() {
        let mut cache: FIFOCache<isize, isize> = FIFOCache::new(3);
        cache.insert(0, 0);
        cache.insert(1, 1);
        cache.insert(2, 2);

        assert!(cache.is_full());
    }

    #[test]
    fn correct_size() {
        let capacity = 100;
        let mut cache: FIFOCache<usize, usize> = FIFOCache::new(capacity);

        for i in 0..capacity {
            cache.insert(i, i);
            assert_eq!(cache.len(), i + 1);
            assert_eq!(cache.rb.len(), cache.hashtable.len());
        }
    }

    #[test]
    fn correct_evict() {
        let capacity = 100;
        let mut cache: FIFOCache<usize, usize> = FIFOCache::new(capacity);

        for i in 0..capacity {
            cache.insert(i, i);
        }

        for i in 0..capacity {
            let value = cache.evict();
            assert!(!value.is_none(), "value should be present.");

            if let Some((key, obj)) = value {
                assert_eq!(key, i);
                assert_eq!(*obj, i);
            }
        }
    }
}
