extern crate priority_queue;
mod landlord;
mod lfu_w;
mod lru;
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_lru_basic_usage() {
        let mut cache = lru::LRUCache::new(3);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.len(), 3);

        // Accessing "a" makes it most recently used
        cache.put("d", 4);
        assert_eq!(cache.get(&"b"), None); // "b" was LRU
    }

    #[test]
    fn test_lfu_unweighted_usage() {
        let mut cache = lfu_w::LFUCache::new(3, false);

        cache.put(1, "apple", None);
        cache.put(2, "banana", None);
        cache.put(3, "cherry", None);

        // Access item 1 multiple times (increases frequency)
        cache.get(&1);
        cache.get(&1);
        cache.get(&1);

        // Access item 2 once
        cache.get(&2);

        // Adding a 4th item evicts the least frequent (key 3)
        cache.put(4, "date", None);

        assert_eq!(cache.get(&3), None); // Evicted (freq=1)
        assert_eq!(cache.get(&1), Some(&"apple")); // Safe (freq=4)
    }

    #[test]
    fn test_lfu_weighted_usage() {
        let mut cache = lfu_w::LFUCache::new(2, true);
        cache.put(1, "low_priority", Some(1));
        cache.put(2, "high_priority", Some(100));

        cache.put(3, "medium_priority", Some(50));
        assert_eq!(cache.get(&1), None); // Evicted due to low weight
        assert_eq!(cache.get(&2), Some(&"high_priority"));
    }

    #[test]
    fn test_all_caches_with_strings() {
        // LRU
        let mut lru = lru::LRUCache::new(2);
        lru.put("hello", "world");
        assert_eq!(lru.get(&"hello"), Some(&"world"));

        // LFU
        let mut lfu = lfu_w::LFUCache::new(2, false);
        lfu.put("foo", "bar", None);
        assert_eq!(lfu.get(&"foo"), Some(&"bar"));

        // Landlord
        let mut landlord = landlord::Landlord::new(2);
        landlord.put("key".to_string(), "value", 10);
        assert_eq!(landlord.get("key".to_string()), Some(&"value"));
    }
}
