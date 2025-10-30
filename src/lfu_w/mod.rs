use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;

use priority_queue::PriorityQueue;

struct Node<K, V> {
    key: K,
    value: V,
    freq: usize,
    weight: usize,
    prev: Option<usize>,
    next: Option<usize>,
}

// impl<K, V> Ord for Node<K, V> {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         return (self.freq * self.weight).cmp(&(other.freq * other.weight));
//     }
// }

// impl<K, V> PartialOrd for Node<K, V> {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl<K, V> PartialEq for Node<K, V> {
//     fn eq(&self, other: &Self) -> bool {
//         return self.freq == other.freq && self.weight == other.weight;
//     }
// }

// impl<K, V> Eq for Node<K, V> {}

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
    weighted: bool,
    nodes: Vec<Node<K, V>>,
    min_priority_queue: priority_queue::PriorityQueue<K, Reverse<usize>>,
    key_to_idx: HashMap<K, usize>,
    priority_to_list: HashMap<usize, PriorityList>,
    free_list: Vec<usize>,
}

impl<K: Clone + Hash + Eq, V> LFUCache<K, V> {
    pub fn new(capacity: usize, weighted: bool) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        LFUCache {
            capacity,
            weighted,
            nodes: Vec::with_capacity(capacity),
            min_priority_queue: PriorityQueue::new(),
            key_to_idx: HashMap::new(),
            priority_to_list: HashMap::new(),
            free_list: Vec::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let idx = *self.key_to_idx.get(key)?;
        self.increment_priority(idx);
        Some(&self.nodes[idx].value)
    }

    pub fn put(&mut self, key: K, value: V, weight: Option<usize>) {
        if self.weighted {
            assert_eq!(true, weight.is_some());
        }
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
            self.add_to_priority_list(idx, 1 * weight.unwrap_or_else(|| 1));
            self.min_priority_queue.push(
                key.clone(),
                std::cmp::Reverse(1 * weight.unwrap_or_else(|| 1)),
            );
        }
    }

    fn increment_priority(&mut self, idx: usize) {
        let weight = self.nodes[idx].weight;
        let old_freq = self.nodes[idx].freq;
        let new_freq = old_freq + 1;

        self.remove_from_priority_list(idx, old_freq * weight);
        self.nodes[idx].freq = new_freq;

        // Add to new priority list
        self.add_to_priority_list(idx, new_freq * weight);
        self.min_priority_queue
            .change_priority(&self.nodes[idx].key, std::cmp::Reverse(new_freq * weight));
    }

    fn add_to_priority_list(&mut self, idx: usize, priority: usize) {
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

    fn remove_from_priority_list(&mut self, idx: usize, priority: usize) {
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
        if let Some(list) = self.priority_to_list.get(&min_priority.1.0) {
            if let Some(tail_idx) = list.tail {
                let key = self.nodes[tail_idx].key.clone();
                self.key_to_idx.remove(&key);
                self.remove_from_priority_list(tail_idx, min_priority.1.0);
                self.free_list.push(tail_idx);
            }
        }
    }

    fn allocate_node(&mut self, key: K, value: V, freq: usize, weight: Option<usize>) -> usize {
        if let Some(free_idx) = self.free_list.pop() {
            self.nodes[free_idx] = Node {
                key,
                value,
                freq,
                weight: weight.unwrap_or_else(|| 1),
                prev: None,
                next: None,
            };
            free_idx
        } else {
            self.nodes.push(Node {
                key,
                value,
                freq,
                weight: weight.unwrap_or_else(|| 1),
                prev: None,
                next: None,
            });
            self.nodes.len() - 1
        }
    }

    pub fn len(&self) -> usize {
        self.key_to_idx.len()
    }

    pub fn is_empty(&self) -> bool {
        self.key_to_idx.is_empty()
    }

    /// Returns the frequency of a key (useful for testing/debugging)
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
        let mut cache = LFUCache::new(2, false);
        cache.put(1, "one", None);
        cache.put(2, "two", None);

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_get_nonexistent() {
        let mut cache: LFUCache<i32, &str> = LFUCache::new(2, false);
        assert_eq!(cache.get(&1), None);
    }

    #[test]
    fn test_update_existing_key() {
        let mut cache = LFUCache::new(2, false);
        cache.put(1, "one", None);
        cache.put(1, "ONE", None);

        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_eviction_on_capacity() {
        let mut cache = LFUCache::new(2, false);
        cache.put(1, "one", None);
        cache.put(2, "two", None);
        cache.put(3, "three", None);

        // Key 1 should be evicted (least frequently used)
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lfu_eviction_with_different_frequencies() {
        let mut cache = LFUCache::new(2, false);
        cache.put(1, "one", None);
        cache.put(2, "two", None);

        // Access key 2 multiple times to increase its frequency
        cache.get(&2);
        cache.get(&2);

        // Add a third item - key 1 should be evicted (lower frequency)
        cache.put(3, "three", None);

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_lru_within_same_frequency() {
        let mut cache = LFUCache::new(2, false);
        cache.put(1, "one", None);
        cache.put(2, "two", None);

        // Both have same frequency (1), so LRU should be evicted
        cache.put(3, "three", None);

        // Key 1 should be evicted (least recently used among freq=1)
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_frequency_tracking() {
        let mut cache = LFUCache::new(3, false);
        cache.put(1, "one", None);

        assert_eq!(cache.get_freq(&1), Some(1));

        cache.get(&1);
        assert_eq!(cache.get_freq(&1), Some(2));

        cache.get(&1);
        assert_eq!(cache.get_freq(&1), Some(3));
    }

    #[test]
    fn test_update_increases_frequency() {
        let mut cache = LFUCache::new(2, false);
        cache.put(1, "one", None);
        cache.put(2, "two", None);

        assert_eq!(cache.get_freq(&1), Some(1));

        // Update key 1
        cache.put(1, "ONE", None);
        assert_eq!(cache.get_freq(&1), Some(2));

        // Key 2 should be evicted (lower frequency)
        cache.put(3, "three", None);

        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_is_empty() {
        let mut cache: LFUCache<i32, &str> = LFUCache::new(2, false);
        assert!(cache.is_empty());

        cache.put(1, "one", None);
        assert!(!cache.is_empty());

        cache.put(2, "two", None);
        cache.put(3, "three", None);
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_single_capacity() {
        let mut cache = LFUCache::new(1, false);
        cache.put(1, "one", None);
        assert_eq!(cache.get(&1), Some(&"one"));

        cache.put(2, "two", None);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
    }

    // ========== Weighted Tests ==========

    #[test]
    fn test_weighted_basic() {
        let mut cache = LFUCache::new(3, true);
        cache.put(1, "one", Some(1));
        cache.put(2, "two", Some(2));

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
    }

    #[test]
    fn test_weighted_eviction_by_priority() {
        let mut cache = LFUCache::new(2, true);
        cache.put(1, "one", Some(1)); // priority = 1 * 1 = 1
        cache.put(2, "two", Some(3)); // priority = 1 * 3 = 3

        // Key 1 has lower priority and should be evicted first
        cache.put(3, "three", Some(1));

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_frequency_increase() {
        let mut cache = LFUCache::new(2, true);
        cache.put(1, "one", Some(2)); // priority = 1 * 2 = 2
        cache.put(2, "two", Some(1)); // priority = 1 * 1 = 1

        // Access key 2, increasing its priority to 2 * 1 = 2
        cache.get(&2);

        // Access key 1, increasing its priority to 2 * 2 = 4
        cache.get(&1);

        // Add third item - key 2 should be evicted (lower priority)
        cache.put(3, "three", Some(1));

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_high_weight_survives() {
        let mut cache = LFUCache::new(2, true);
        cache.put(1, "one", Some(10)); // priority = 1 * 10 = 10
        cache.put(2, "two", Some(1)); // priority = 1 * 1 = 1
        cache.put(3, "three", Some(1)); // priority = 1 * 1 = 1

        // Key 1 should survive due to high weight
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_equal_priority_lru() {
        let mut cache = LFUCache::new(2, true);
        cache.put(1, "one", Some(2)); // priority = 1 * 2 = 2
        cache.put(2, "two", Some(2)); // priority = 1 * 2 = 2

        // Both have same priority, LRU should be evicted
        cache.put(3, "three", Some(1));

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_update_maintains_weight() {
        let mut cache = LFUCache::new(2, true);
        cache.put(1, "one", Some(5)); // priority = 1 * 5 = 5

        // Update should maintain weight and increase frequency
        cache.put(1, "ONE", Some(5)); // priority = 2 * 5 = 10

        cache.put(2, "two", Some(1)); // priority = 1 * 1 = 1

        // Add third item - key 2 should be evicted
        cache.put(3, "three", Some(2)); // priority = 1 * 2 = 2

        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_weighted_complex_scenario() {
        let mut cache = LFUCache::new(3, true);
        cache.put(1, "one", Some(1)); // priority = 1 * 1 = 1
        cache.put(2, "two", Some(2)); // priority = 1 * 2 = 2
        cache.put(3, "three", Some(3)); // priority = 1 * 3 = 3

        // Access patterns
        cache.get(&1); // priority = 2 * 1 = 2
        cache.get(&1); // priority = 3 * 1 = 3
        cache.get(&2); // priority = 2 * 2 = 4

        // At this point: key1=3, key2=4, key3=3
        // key3 is LRU among priority=3

        cache.put(4, "four", Some(1)); // priority = 1 * 1 = 1

        // Key 4 should evict key 3 (lowest priority)
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&4), Some(&"four"));
    }

    #[test]
    #[should_panic(expected = "Capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        let _cache: LFUCache<i32, &str> = LFUCache::new(0, false);
    }

    // ========== Large Capacity Tests ==========

    #[test]
    fn test_large_capacity() {
        let mut cache = LFUCache::new(100, false);
        for i in 0..100 {
            cache.put(i, i * 2, None);
        }

        assert_eq!(cache.len(), 100);

        // Access some keys
        for i in 0..50 {
            assert_eq!(cache.get(&i), Some(&(i * 2)));
        }

        // Add more items
        for i in 100..150 {
            cache.put(i, i * 2, None);
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
