# search_engine_cache

A high-performance collection of cache implementations in Rust, featuring LRU, LFU, and Landlord eviction policies.

## Cache Types

### LRU Cache (Least Recently Used)
Evicts the least recently accessed items first. Perfect for general-purpose caching where recent access patterns predict future access.

**Use when:**
- You want simple, predictable behavior
- Recent items are likely to be accessed again
- Access recency matters more than frequency

### LFU Cache (Least Frequently Used)
Evicts items based on access frequency, with optional weighted support for priority-based eviction.

**Use when:**
- Popular items should stay cached longer
- Access frequency is a better predictor than recency
- Items have different importance levels (weighted mode)

### Landlord Cache
A weight-based cache with dynamic priority updates. Priority increases on access and items are evicted based on lowest priority.

**Use when:**
- Items have different costs or sizes
- You need fine-grained control over eviction priorities
- Cache efficiency optimization is critical

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
search_engine_cache = "0.1.0"
```

## Quick Start

### LRU Cache

```rust
use search_engine_cache::lru::LRUCache;

fn main() {
    let mut cache = LRUCache::new(3);
    
    cache.put(1, "apple");
    cache.put(2, "banana");
    cache.put(3, "cherry");
    
    // Access an item (makes it "recently used")
    cache.get(&1);
    
    // Adding a 4th item evicts the least recently used (key 2)
    cache.put(4, "date");
    
    assert_eq!(cache.get(&2), None);  // Evicted
    assert_eq!(cache.get(&1), Some(&"apple"));  // Still present
}
```

### LFU Cache (Unweighted)

```rust
use search_engine_cache::lfu_w::LFUCache;

fn main() {
    let mut cache = LFUCache::new(3, false);
    
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
    
    assert_eq!(cache.get(&3), None);  // Evicted (freq=1)
    assert_eq!(cache.get(&1), Some(&"apple"));  // Safe (freq=4)
}
```