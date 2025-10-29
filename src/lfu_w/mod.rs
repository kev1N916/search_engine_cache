use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

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
    min_priority: usize,
    nodes: Vec<Node<K, V>>,
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
            min_priority: 0,
            nodes: Vec::with_capacity(capacity),
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
            self.key_to_idx.insert(key, idx);
            self.add_to_priority_list(idx, 1*weight.unwrap_or_else(|| 1));
            self.min_priority = 1*weight.unwrap_or_else(|| 1);
        }
    }

    fn increment_priority(&mut self, idx: usize) {
        let weight=self.nodes[idx].weight;
        let old_freq = self.nodes[idx].freq;
        let new_freq = old_freq + 1;

        self.remove_from_priority_list(idx, old_freq*weight);

        // Update node frequency
        self.nodes[idx].freq = new_freq;

        // Add to new frequency list
        self.add_to_priority_list(idx, new_freq*weight);

        // Update min_freq if necessary
        if old_freq*weight == self.min_priority {
            if let Some(list) = self.priority_to_list.get(&(old_freq*weight)) {
                if list.size == 0 {
                    self.min_priority = new_freq*weight;
                }
            }
        }
    }

    fn add_to_priority_list(&mut self, idx: usize, freq: usize) {
        let list = self
            .priority_to_list
            .entry(freq)
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
        if let Some(list) = self.priority_to_list.get(&self.min_priority) {
            if let Some(tail_idx) = list.tail {
                let key = self.nodes[tail_idx].key.clone();
                self.key_to_idx.remove(&key);
                self.remove_from_priority_list(tail_idx, self.min_priority);
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

    #[test]
    fn test_basic_operations() {
        let mut cache = LFUCache::new(2, false);

        cache.put(1, "one", None);
        cache.put(2, "two", None);

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
    }

    #[test]
    fn test_lfu_eviction() {
        let mut cache = LFUCache::new(2, false);

        cache.put(1, "one", None); // freq: 1
        cache.put(2, "two", None); // freq: 1
        cache.get(&1); // freq: 2
        cache.put(3, "three", None); // Should evict key 2 (freq: 1)

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), None); // Evicted
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_frequency_tracking() {
        let mut cache = LFUCache::new(3, false);

        cache.put(1, "one", None);
        cache.put(2, "two", None);
        cache.put(3, "three", None);

        cache.get(&1);
        cache.get(&1);
        cache.get(&2);

        assert_eq!(cache.get_freq(&1), Some(3)); // 1 put + 2 gets
        assert_eq!(cache.get_freq(&2), Some(2)); // 1 put + 1 get
        assert_eq!(cache.get_freq(&3), Some(1)); // 1 put
    }

    #[test]
    fn test_same_freq_lru_order() {
        let mut cache = LFUCache::new(2, false);

        cache.put(1, "one", None);
        cache.put(2, "two", None);
        cache.put(3, "three", None);

        assert_eq!(cache.get(&1), None); // Evicted (LRU within same frequency)
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_update_existing_key() {
        let mut cache = LFUCache::new(2, false);

        cache.put(1, "one", None);
        cache.get(&1);
        cache.put(1, "ONE", None); // Update, should increment frequency

        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.get_freq(&1), Some(4)); // put + get + put + get
    }

    #[test]
    fn test_complex_scenario() {
        let mut cache = LFUCache::new(3, false);

        cache.put(1, 1, None);
        cache.put(2, 2, None);
        cache.put(3, 3, None);

        cache.get(&1); // 1: freq=2
        cache.get(&1); // 1: freq=3
        cache.get(&2); // 2: freq=2

        cache.put(4, 4, None); // Should evict 3 (freq=1)

        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&1), Some(&1));
        assert_eq!(cache.get(&2), Some(&2));
        assert_eq!(cache.get(&4), Some(&4));

        cache.put(5, 5, None); // Should evict 4 (freq=2, but most recent among freq=2)

        assert_eq!(cache.get(&4), None);
    }

    #[test]
    fn test_capacity_one() {
        let mut cache = LFUCache::new(1, false);

        cache.put(1, "one", None);
        assert_eq!(cache.get(&1), Some(&"one"));

        cache.put(2, "two", None); // Should evict 1
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
    }
}
