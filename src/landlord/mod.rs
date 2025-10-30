use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::HashMap;

use std::hash::Hash;

pub struct LandlordNode<V> {
    value: V,
    weight: u32,
}

pub struct Landlord<K, V> {
    capacity: usize,
    l: u32,
    pq: PriorityQueue<K, Reverse<u32>>,
    cache: HashMap<K, LandlordNode<V>>,
}

impl<K: Clone + Hash + Eq, V> Landlord<K, V> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        Landlord {
            capacity,
            l: 0,
            pq: PriorityQueue::new(),
            cache: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: K, value: V, weight: u32) {
        if self.cache.contains_key(&key){
            self.remove(&key);
        }
        if self.cache.len() >= self.capacity {
            self.evict();
        }
        self.cache.insert(
            key.clone(),
            LandlordNode {
                value: value,
                weight: weight,
            },
        );
        self.pq.push(key.clone(), Reverse(self.l + weight));
    }

    fn remove(&mut self,key: &K){
        self.cache.remove(&key);
        self.pq.remove(key);
    }
    pub fn get(&mut self, key: K) -> Option<&V> {
        if let Some(landlord_node) = self.cache.get(&key) {
            let new_priority = self.l + landlord_node.weight;
            self.pq.change_priority(&key, Reverse(new_priority));
            Some(&landlord_node.value)
        } else {
            None
        }
    }

    pub fn evict(&mut self) {
        if let Some(evicted_key) = self.pq.pop() {
            self.l = evicted_key.1.0;
            self.cache.remove(&evicted_key.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let cache: Landlord<String, i32> = Landlord::new(5);
        assert_eq!(cache.capacity, 5);
        assert_eq!(cache.l, 0);
    }

    #[test]
    #[should_panic(expected = "Capacity must be greater than 0")]
    fn test_new_cache_zero_capacity() {
        let _cache: Landlord<String, i32> = Landlord::new(0);
    }

    #[test]
    fn test_put_and_get() {
        let mut cache = Landlord::new(3);
        cache.put("key1".to_string(), 100, 10);

        let value = cache.get("key1".to_string());
        assert_eq!(value, Some(&100));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let mut cache: Landlord<String, i32> = Landlord::new(3);
        let value = cache.get("nonexistent".to_string());
        assert_eq!(value, None);
    }

    #[test]
    fn test_put_multiple_items() {
        let mut cache = Landlord::new(3);
        cache.put("key1".to_string(), 100, 10);
        cache.put("key2".to_string(), 200, 20);
        cache.put("key3".to_string(), 300, 30);

        assert_eq!(cache.get("key1".to_string()), Some(&100));
        assert_eq!(cache.get("key2".to_string()), Some(&200));
        assert_eq!(cache.get("key3".to_string()), Some(&300));
    }

    #[test]
    fn test_eviction_on_capacity_reached() {
        let mut cache = Landlord::new(2);
        cache.put("key1".to_string(), 100, 10);
        cache.put("key2".to_string(), 200, 20);

        // This should trigger eviction of key1 (lowest priority)
        cache.put("key3".to_string(), 300, 30);

        assert_eq!(cache.get("key1".to_string()), None);
        assert_eq!(cache.get("key2".to_string()), Some(&200));
        assert_eq!(cache.get("key3".to_string()), Some(&300));
    }

    #[test]
    fn test_priority_update_on_get() {
        let mut cache = Landlord::new(2);
        cache.put("key1".to_string(), 100, 10);
        cache.put("key2".to_string(), 200, 5);

        // Access key1 to boost its priority
        cache.get("key1".to_string());

        // Adding key3 should evict key2 (lowest priority after key1 was accessed)
        cache.put("key3".to_string(), 300, 15);

        assert_eq!(cache.get("key1".to_string()), Some(&100));
        assert_eq!(cache.get("key2".to_string()), None);
        assert_eq!(cache.get("key3".to_string()), Some(&300));
    }

    #[test]
    fn test_l_value_updates_on_eviction() {
        let mut cache = Landlord::new(2);
        cache.put("key1".to_string(), 100, 10);
        assert_eq!(cache.l, 0);

        cache.put("key2".to_string(), 200, 20);
        cache.put("key3".to_string(), 300, 30); // Triggers eviction

        // After eviction, l should be updated to the priority of the evicted item
        assert_eq!(cache.l, 10);
    }

    #[test]
    fn test_manual_evict() {
        let mut cache = Landlord::new(3);
        cache.put("key1".to_string(), 100, 10);
        cache.put("key2".to_string(), 200, 20);

        cache.evict();

        assert_eq!(cache.get("key1".to_string()), None);
        assert_eq!(cache.get("key2".to_string()), Some(&200));
    }

    #[test]
    fn test_evict_on_empty_cache() {
        let mut cache: Landlord<String, i32> = Landlord::new(3);
        cache.evict(); // Should not panic
        assert_eq!(cache.l, 0);
    }

    #[test]
    fn test_weight_based_eviction() {
        let mut cache = Landlord::new(3);
        cache.put("low_weight".to_string(), 1, 5);
        cache.put("high_weight".to_string(), 2, 50);
        cache.put("medium_weight".to_string(), 3, 25);

        // Adding another item should evict low_weight (lowest priority)
        cache.put("new_item".to_string(), 4, 30);

        assert_eq!(cache.get("low_weight".to_string()), None);
        assert_eq!(cache.get("high_weight".to_string()), Some(&2));
        assert_eq!(cache.get("medium_weight".to_string()), Some(&3));
        assert_eq!(cache.get("new_item".to_string()), Some(&4));
    }

    #[test]
    fn test_overwrite_existing_key() {
        let mut cache = Landlord::new(3);
        cache.put("key1".to_string(), 100, 10);
        cache.put("key1".to_string(), 200, 20);

        assert_eq!(cache.get("key1".to_string()), Some(&200));
    }

    #[test]
    fn test_with_integer_keys() {
        let mut cache = Landlord::new(3);
        cache.put(1, "value1", 10);
        cache.put(2, "value2", 20);
        cache.put(3, "value3", 30);

        assert_eq!(cache.get(1), Some(&"value1"));
        assert_eq!(cache.get(2), Some(&"value2"));
        assert_eq!(cache.get(3), Some(&"value3"));
    }

    #[test]
    fn test_sequential_evictions() {
        let mut cache = Landlord::new(2);
        cache.put("key1".to_string(), 100, 10);
        cache.put("key2".to_string(), 200, 20);

        let initial_l = cache.l;
        cache.put("key3".to_string(), 300, 30);
        assert!(cache.l > initial_l);

        let second_l = cache.l;
        cache.put("key4".to_string(), 400, 40);
        assert!(cache.l > second_l);
    }
}
