extern crate priority_queue;
pub mod landlord;
pub mod lfu_w;
pub mod lru;

use std::hash::Hash;

use landlord::Landlord;
use lfu_w::LFUCache;
use lru::LRUCache;
pub enum CacheType<K, V> {
    LRU(LRUCache<K, V>),
    LFU(LFUCache<K, V>),
    Landlord(Landlord<K, V>),
}

impl<K: Clone + Hash + Eq, V> CacheType<K, V> {
    pub fn new_lru(capacity: usize) -> Self {
        CacheType::LRU(LRUCache::new(capacity))
    }

    pub fn new_lfu(capacity: usize) -> Self {
        CacheType::LFU(LFUCache::new(capacity))
    }

    pub fn new_landlord(capacity: usize) -> Self {
        CacheType::Landlord(Landlord::new(capacity))
    }
}

impl<K: Clone + Hash + Eq, V> CacheType<K, V> {
    pub fn put(&mut self, key: K, value: V, weight: u32) {
        match self {
            CacheType::LRU(cache) => cache.put(key, value, weight),
            CacheType::LFU(cache) => cache.put(key, value, weight),
            CacheType::Landlord(cache) => cache.put(key, value, weight),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        match self {
            CacheType::LRU(cache) => cache.get(key),
            CacheType::LFU(cache) => cache.get(key),
            CacheType::Landlord(cache) => cache.get(key),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            CacheType::LRU(cache) => cache.len(),
            CacheType::LFU(cache) => cache.len(),
            CacheType::Landlord(cache) => cache.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            CacheType::LRU(cache) => cache.is_empty(),
            CacheType::LFU(cache) => cache.is_empty(),
            CacheType::Landlord(cache) => cache.is_empty(),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_lru_basic_usage() {
        let mut cache = lru::LRUCache::new(3);
        cache.put("a", 1, 0);
        cache.put("b", 2, 0);
        cache.put("c", 3, 0);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.len(), 3);

        // Accessing "a" makes it most recently used
        cache.put("d", 4, 0);
        assert_eq!(cache.get(&"b"), None); // "b" was LRU
    }

    #[test]
    fn test_lfu_unweighted_usage() {
        let mut cache = lfu_w::LFUCache::new(3);

        cache.put(1, "apple", 1);
        cache.put(2, "banana", 1);
        cache.put(3, "cherry", 1);

        // Access item 1 multiple times (increases frequency)
        cache.get(&1);
        cache.get(&1);
        cache.get(&1);

        // Access item 2 once
        cache.get(&2);

        // Adding a 4th item evicts the least frequent (key 3)
        cache.put(4, "date", 1);

        assert_eq!(cache.get(&3), None); // Evicted (freq=1)
        assert_eq!(cache.get(&1), Some(&"apple")); // Safe (freq=4)
    }

    #[test]
    fn test_lfu_weighted_usage() {
        let mut cache = lfu_w::LFUCache::new(2);
        cache.put(1, "low_priority", 1);
        cache.put(2, "high_priority", 100);

        cache.put(3, "medium_priority", 50);
        assert_eq!(cache.get(&1), None); // Evicted due to low weight
        assert_eq!(cache.get(&2), Some(&"high_priority"));
    }

    #[test]
    fn test_all_caches_with_strings() {
        // LRU
        let mut lru = lru::LRUCache::new(2);

        lru.put("hello", "world", 0);
        assert_eq!(lru.get(&"hello"), Some(&"world"));

        // LFU
        let mut lfu = lfu_w::LFUCache::new(2);

        lfu.put("foo", "bar", 1);
        assert_eq!(lfu.get(&"foo"), Some(&"bar"));

        // Landlord
        let mut landlord = landlord::Landlord::new(2);

        landlord.put("key".to_string(), "value", 10);
        assert_eq!(landlord.get(&"key".to_string()), Some(&"value"));
    }
}
