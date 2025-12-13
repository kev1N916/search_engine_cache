use std::collections::HashMap;
use std::hash::Hash;

use crate::Cache;

struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct LRUCache<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>,
    head: Option<usize>,
    tail: Option<usize>,
    free_list: Vec<usize>,
}

impl<K: Clone + Hash + Eq, V> Cache<K, V> for LRUCache<K, V> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        LRUCache {
            capacity,
            map: HashMap::new(),
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            free_list: Vec::new(),
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        let idx = *self.map.get(key)?;
        self.move_to_front(idx);
        Some(&self.nodes[idx].value)
    }

    fn put(&mut self, key: K, value: V, _weight: u32) {
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            self.move_to_front(idx);
        } else {
            // Need to evict if at capacity
            if self.map.len() >= self.capacity {
                self.remove_tail();
            }

            // Get index for new node
            let idx = if let Some(free_idx) = self.free_list.pop() {
                self.nodes[free_idx] = Node {
                    key: key.clone(),
                    value,
                    prev: None,
                    next: None,
                };
                free_idx
            } else {
                self.nodes.push(Node {
                    key: key.clone(),
                    value,
                    prev: None,
                    next: None,
                });
                self.nodes.len() - 1
            };

            self.map.insert(key, idx);
            self.add_to_front(idx);
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl<K: Clone + Hash + Eq, V> LRUCache<K, V> {
    fn move_to_front(&mut self, idx: usize) {
        if self.head == Some(idx) {
            return;
        }

        self.detach(idx);
        self.add_to_front(idx);
    }

    fn detach(&mut self, idx: usize) {
        let node = &self.nodes[idx];
        let prev = node.prev;
        let next = node.next;

        match prev {
            Some(p) => self.nodes[p].next = next,
            None => self.head = next,
        }

        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => self.tail = prev,
        }
    }

    fn add_to_front(&mut self, idx: usize) {
        self.nodes[idx].prev = None;
        self.nodes[idx].next = self.head;

        if let Some(old_head) = self.head {
            self.nodes[old_head].prev = Some(idx);
        }

        self.head = Some(idx);

        if self.tail.is_none() {
            self.tail = Some(idx);
        }
    }

    fn remove_tail(&mut self) {
        if let Some(tail_idx) = self.tail {
            let key = self.nodes[tail_idx].key.clone();
            self.map.remove(&key);
            self.detach(tail_idx);
            self.free_list.push(tail_idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "one", 0);
        cache.put(2, "two", 0);

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
    }

    #[test]
    fn test_eviction() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "one", 0);
        cache.put(2, "two", 0);
        cache.put(3, "three", 0);

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_lru_order() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "one", 0);
        cache.put(2, "two", 0);
        cache.get(&1);
        cache.put(3, "three", 0);

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some(&"three"));
    }
}
