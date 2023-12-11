use std::cmp::min;
use std::collections::HashMap;
use std::hash::Hash;

use super::ring_buffer::RingBuffer;

#[derive(Default, Clone)]
pub struct CacheMetadata {
    freq: usize,
}

impl CacheMetadata {
    #[inline(always)]
    fn inc_freq(&mut self) {
        self.freq = min(self.freq + 1, 3);
    }
}

#[derive(Default, Clone)]
pub struct CacheObject<K, V> {
    key: K,
    value: V,
    meta: CacheMetadata
}

pub struct FIFOCache<K, V> {
    rb: RingBuffer<CacheObject<K, V>>,
    hashtable: HashMap<K, V>,
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

    // TODO: Result return type to indicate potential eviction problem.
    pub fn evict(&mut self) {
        let obj = self.rb.pop_front();
        if let Some(obj) = obj {
            let k = obj.key;
            self.hashtable.remove(&k);
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.hashtable.insert(key.clone(), value.clone());
        self.rb.push_back(
            CacheObject { key, value, meta: Default::default() }
        );
    }

}

impl<K, V> FIFOCache<K, V>
where K: Eq + Hash
{
    // Separate impl block more generic trait bound
    #[inline]
    pub fn find(&self, key: &K) -> Option<&V> {
        self.hashtable.get(key)
    }
}

impl<K, V> FIFOCache<K, V> {
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.rb.is_full()
    }
}
