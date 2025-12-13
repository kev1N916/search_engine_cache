use std::cmp::Reverse;
use std::collections::HashMap;
use std::hash::Hash;

use priority_queue::PriorityQueue;

use crate::Cache;

struct Node<K, V> {
    key: K,
    value: V,
    freq: usize,
    weight: u32,
    prev: Option<usize>,
    next: Option<usize>,
}

struct PriorityList {
    head: Option<usize>,
    tail: Option<usize>,
    size: usize,
}

impl PriorityList {
    fn new() -> Self {
        PriorityList {
            head: None,
            tail: None,
            size: 0,
        }
    }
}

pub struct LFUCache<K, V> {
    capacity: usize,
    nodes: Vec<Node<K, V>>,
    min_priority_queue: priority_queue::PriorityQueue<K, Reverse<u32>>,
    key_to_idx: HashMap<K, usize>,
    priority_to_list: HashMap<u32, PriorityList>,
    free_list: Vec<usize>,
}

impl<K: Clone + Hash + Eq, V> Cache<K, V> for LFUCache<K, V> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        LFUCache {
            capacity,
            nodes: Vec::with_capacity(capacity),
            min_priority_queue: PriorityQueue::new(),
            key_to_idx: HashMap::new(),
            priority_to_list: HashMap::new(),
            free_list: Vec::new(),
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        let idx = *self.key_to_idx.get(key)?;
        self.increment_priority(idx);
        Some(&self.nodes[idx].value)
    }

    fn put(&mut self, key: K, value: V, weight: u32) {
        if let Some(&idx) = self.key_to_idx.get(&key) {
            // Update existing key
            self.nodes[idx].value = value;
            self.increment_priority(idx);
        } else {
            // Need to evict if at capacity
            if self.key_to_idx.len() >= self.capacity {
                self.evict_lfu();
            }
            // Create new node with frequency 1
            let idx = self.allocate_node(key.clone(), value, 1, weight);
            self.key_to_idx.insert(key.clone(), idx);
            self.add_to_priority_list(idx, 1 * weight);
            self.min_priority_queue
                .push(key.clone(), std::cmp::Reverse(1 * weight));
        }
    }

    fn len(&self) -> usize {
        self.key_to_idx.len()
    }

    fn is_empty(&self) -> bool {
        self.key_to_idx.is_empty()
    }
}

impl<K: Clone + Hash + Eq, V> LFUCache<K, V> {
    fn increment_priority(&mut self, idx: usize) {
        let weight = self.nodes[idx].weight;
        let old_freq = self.nodes[idx].freq;
        let new_freq = old_freq + 1;

        self.remove_from_priority_list(idx, old_freq as u32 * weight);
        self.nodes[idx].freq = new_freq;

        // Add to new priority list
        self.add_to_priority_list(idx, new_freq as u32 * weight);
        self.min_priority_queue.change_priority(
            &self.nodes[idx].key,
            std::cmp::Reverse(new_freq as u32 * weight),
        );
    }

    fn add_to_priority_list(&mut self, idx: usize, priority: u32) {
        let list = self
            .priority_to_list
            .entry(priority)
            .or_insert_with(PriorityList::new);

        self.nodes[idx].next = list.head;
        self.nodes[idx].prev = None;

        if let Some(old_head) = list.head {
            self.nodes[old_head].prev = Some(idx);
        }

        list.head = Some(idx);

        if list.tail.is_none() {
            list.tail = Some(idx);
        }

        list.size += 1;
    }

    fn remove_from_priority_list(&mut self, idx: usize, priority: u32) {
        let node = &self.nodes[idx];
        let prev = node.prev;
        let next = node.next;

        if let Some(list) = self.priority_to_list.get_mut(&priority) {
            match prev {
                Some(p) => self.nodes[p].next = next,
                None => list.head = next,
            }

            match next {
                Some(n) => self.nodes[n].prev = prev,
                None => list.tail = prev,
            }

            list.size -= 1;
        }
    }

    fn evict_lfu(&mut self) {
        // Remove the tail (least recently used) from min frequency list
        let min_priority = self.min_priority_queue.pop().unwrap();
        if let Some(list) = self.priority_to_list.get(&min_priority.1 .0) {
            if let Some(tail_idx) = list.tail {
                let key = self.nodes[tail_idx].key.clone();
                self.key_to_idx.remove(&key);
                self.remove_from_priority_list(tail_idx, min_priority.1 .0);
                self.free_list.push(tail_idx);
            }
        }
    }

    fn allocate_node(&mut self, key: K, value: V, freq: usize, weight: u32) -> usize {
        if let Some(free_idx) = self.free_list.pop() {
            self.nodes[free_idx] = Node {
                key,
                value,
                freq,
                weight,
                prev: None,
                next: None,
            };
            free_idx
        } else {
            self.nodes.push(Node {
                key,
                value,
                freq,
                weight,
                prev: None,
                next: None,
            });
            self.nodes.len() - 1
        }
    }

    pub fn get_freq(&self, key: &K) -> Option<usize> {
        self.key_to_idx.get(key).map(|&idx| self.nodes[idx].freq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Basic Unweighted Tests ==========

    #[test]
    fn test_basic_put_and_get() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 1);
        cache.put(2, "two", 1);

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_get_nonexistent() {
        let mut cache: LFUCache<i32, &str> = LFUCache::new(2);
        assert_eq!(cache.get(&1), None);
    }

    #[test]
    fn test_update_existing_key() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 1);
        cache.put(1, "ONE", 1);

        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_eviction_on_capacity() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 1);
        cache.put(2, "two", 1);
        cache.put(3, "three", 1);

        // Key 1 should be evicted (least frequently used)
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lfu_eviction_with_different_frequencies() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 1);
        cache.put(2, "two", 1);

        // Access key 2 multiple times to increase its frequency
        cache.get(&2);
        cache.get(&2);

        // Add a third item - key 1 should be evicted (lower frequency)
        cache.put(3, "three", 1);

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_lru_within_same_frequency() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 1);
        cache.put(2, "two", 1);

        // Both have same frequency (1), so LRU should be evicted
        cache.put(3, "three", 1);

        // Key 1 should be evicted (least recently used among freq=1)
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_frequency_tracking() {
        let mut cache = LFUCache::new(3);
        cache.put(1, "one", 1);

        assert_eq!(cache.get_freq(&1), Some(1));

        cache.get(&1);
        assert_eq!(cache.get_freq(&1), Some(2));

        cache.get(&1);
        assert_eq!(cache.get_freq(&1), Some(3));
    }

    #[test]
    fn test_update_increases_frequency() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 1);
        cache.put(2, "two", 1);

        assert_eq!(cache.get_freq(&1), Some(1));

        // Update key 1
        cache.put(1, "ONE", 1);
        assert_eq!(cache.get_freq(&1), Some(2));

        // Key 2 should be evicted (lower frequency)
        cache.put(3, "three", 1);

        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_is_empty() {
        let mut cache = LFUCache::new(2);
        assert!(cache.is_empty());

        cache.put(1, "one", 1);
        assert!(!cache.is_empty());

        cache.put(2, "two", 1);
        cache.put(3, "three", 1);
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_single_capacity() {
        let mut cache = LFUCache::new(1);
        cache.put(1, "one", 1);
        assert_eq!(cache.get(&1), Some(&"one"));

        cache.put(2, "two", 1);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
    }

    // ========== Weighted Tests ==========

    #[test]
    fn test_weighted_basic() {
        let mut cache = LFUCache::new(3);
        cache.put(1, "one", 1);
        cache.put(2, "two", 2);

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
    }

    #[test]
    fn test_weighted_eviction_by_priority() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 1); // priority = 1 * 1 = 1
        cache.put(2, "two", 3); // priority = 1 * 3 = 3

        // Key 1 has lower priority and should be evicted first
        cache.put(3, "three", 1);

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_frequency_increase() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 2); // priority = 1 * 2 = 2
        cache.put(2, "two", 1); // priority = 1 * 1 = 1

        // Access key 2, increasing its priority to 2 * 1 = 2
        cache.get(&2);

        // Access key 1, increasing its priority to 2 * 2 = 4
        cache.get(&1);

        // Add third item - key 2 should be evicted (lower priority)
        cache.put(3, "three", 1);

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_high_weight_survives() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 10); // priority = 1 * 10 = 10
        cache.put(2, "two", 1); // priority = 1 * 1 = 1
        cache.put(3, "three", 1); // priority = 1 * 1 = 1

        // Key 1 should survive due to high weight
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_equal_priority_lru() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 2); // priority = 1 * 2 = 2
        cache.put(2, "two", 2); // priority = 1 * 2 = 2

        // Both have same priority, LRU should be evicted
        cache.put(3, "three", 1);

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_update_maintains_weight() {
        let mut cache = LFUCache::new(2);
        cache.put(1, "one", 5); // priority = 1 * 5 = 5

        // Update should maintain weight and increase frequency
        cache.put(1, "ONE", 5); // priority = 2 * 5 = 10

        cache.put(2, "two", 1); // priority = 1 * 1 = 1

        // Add third item - key 2 should be evicted
        cache.put(3, "three", 2); // priority = 1 * 2 = 2

        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_complex_scenario() {
        let mut cache = LFUCache::new(3);
        cache.put(1, "one", 1); // priority = 1 * 1 = 1
        cache.put(2, "two", 2); // priority = 1 * 2 = 2
        cache.put(3, "three", 3); // priority = 1 * 3 = 3

        // Access patterns
        cache.get(&1); // priority = 2 * 1 = 2
        cache.get(&1); // priority = 3 * 1 = 3
        cache.get(&2); // priority = 2 * 2 = 4

        // At this point: key1=3, key2=4, key3=3
        // key3 is LRU among priority=3

        cache.put(4, "four", 1); // priority = 1 * 1 = 1

        // Key 4 should evict key 3 (lowest priority)
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&4), Some(&"four"));
    }

    #[test]
    #[should_panic(expected = "Capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        let _cache: LFUCache<i32, &str> = LFUCache::new(0);
    }

    // ========== Large Capacity Tests ==========

    #[test]
    fn test_large_capacity() {
        let mut cache = LFUCache::new(100);
        for i in 0..100 {
            cache.put(i, i * 2, 1);
        }

        assert_eq!(cache.len(), 100);

        // Access some keys
        for i in 0..50 {
            assert_eq!(cache.get(&i), Some(&(i * 2)));
        }

        // Add more items
        for i in 100..150 {
            cache.put(i, i * 2, 1);
        }

        // First 50 should still be there (higher frequency)
        for i in 0..50 {
            assert_eq!(cache.get(&i), Some(&(i * 2)));
        }

        // Items 50-99 should have been evicted
        for i in 50..100 {
            assert_eq!(cache.get(&i), None);
        }
    }
}
