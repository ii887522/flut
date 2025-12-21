use flut::collections::lru_cache::LruCache;

#[test]
fn test_new() {
  let cache: LruCache<i32> = LruCache::new();
  // Since there are no public methods to check internal state,
  // we verify that creation doesn't panic
  drop(cache);
}

#[test]
fn test_default() {
  let cache: LruCache<String> = LruCache::default();
  drop(cache);
}

#[test]
fn test_with_capacity() {
  let cache: LruCache<i32> = LruCache::with_capacity(100);
  drop(cache);
}

#[test]
fn test_with_capacity_zero() {
  let cache: LruCache<i32> = LruCache::with_capacity(0);
  drop(cache);
}

#[test]
fn test_add_single_item() {
  let mut cache = LruCache::new();
  cache.add(42);
  // Item added successfully - no panic
}

#[test]
fn test_add_multiple_items() {
  let mut cache = LruCache::new();
  cache.add(1);
  cache.add(2);
  cache.add(3);
  cache.add(4);
  cache.add(5);
}

#[test]
fn test_add_duplicate_items() {
  let mut cache = LruCache::new();
  cache.add(42);
  cache.add(42);
  cache.add(42);
  // Adding duplicates should work (they get different queue numbers)
}

#[test]
fn test_add_strings() {
  let mut cache = LruCache::new();
  cache.add("hello".to_string());
  cache.add("world".to_string());
  cache.add("test".to_string());
}

#[test]
fn test_invalidate_one_empty_cache() {
  let mut cache: LruCache<i32> = LruCache::new();
  let result = cache.invalidate_one();
  assert_eq!(result, None);
}

#[test]
fn test_invalidate_one_single_item() {
  let mut cache = LruCache::new();
  cache.add(42);
  let result = cache.invalidate_one();
  assert_eq!(result, Some(42));
}

#[test]
fn test_invalidate_one_multiple_items() {
  let mut cache = LruCache::new();
  cache.add(1);
  cache.add(2);
  cache.add(3);

  // Should remove the oldest item (FIFO order from BTreeMap)
  let first = cache.invalidate_one();
  assert_eq!(first, Some(1));

  let second = cache.invalidate_one();
  assert_eq!(second, Some(2));

  let third = cache.invalidate_one();
  assert_eq!(third, Some(3));

  let fourth = cache.invalidate_one();
  assert_eq!(fourth, None);
}

#[test]
fn test_invalidate_one_lru_order() {
  let mut cache = LruCache::new();
  cache.add("first".to_string());
  cache.add("second".to_string());
  cache.add("third".to_string());

  // Items should be invalidated in the order they were added (oldest first)
  assert_eq!(cache.invalidate_one(), Some("first".to_string()));
  assert_eq!(cache.invalidate_one(), Some("second".to_string()));
  assert_eq!(cache.invalidate_one(), Some("third".to_string()));
  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_remove_existing_item() {
  let mut cache = LruCache::new();
  cache.add(10);
  cache.add(20);
  cache.add(30);

  cache.remove(&20);

  // Verify that other items can still be invalidated
  let first = cache.invalidate_one();
  assert!(first == Some(10) || first == Some(30));

  let second = cache.invalidate_one();
  assert!(second == Some(10) || second == Some(30));
  assert_ne!(first, second);

  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_remove_non_existing_item() {
  let mut cache = LruCache::new();
  cache.add(10);
  cache.add(20);

  // Removing non-existing item should not panic
  cache.remove(&99);

  // Original items should still be there
  assert!(cache.invalidate_one().is_some());
  assert!(cache.invalidate_one().is_some());
}

#[test]
fn test_remove_from_empty_cache() {
  let mut cache: LruCache<i32> = LruCache::new();
  cache.remove(&42);
  // Should not panic
}

#[test]
fn test_remove_all_items() {
  let mut cache = LruCache::new();
  cache.add(1);
  cache.add(2);
  cache.add(3);

  cache.remove(&1);
  cache.remove(&2);
  cache.remove(&3);

  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_remove_same_item_twice() {
  let mut cache = LruCache::new();
  cache.add(42);

  cache.remove(&42);
  cache.remove(&42); // Second remove should be no-op

  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_add_after_remove() {
  let mut cache = LruCache::new();
  cache.add(1);
  cache.add(2);

  cache.remove(&1);

  cache.add(3);
  cache.add(4);

  // Should have items 2, 3, 4
  assert_eq!(cache.invalidate_one(), Some(2));
  assert_eq!(cache.invalidate_one(), Some(3));
  assert_eq!(cache.invalidate_one(), Some(4));
  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_add_after_invalidate() {
  let mut cache = LruCache::new();
  cache.add(1);
  cache.add(2);

  cache.invalidate_one();

  cache.add(3);

  // Should have items 2, 3
  assert_eq!(cache.invalidate_one(), Some(2));
  assert_eq!(cache.invalidate_one(), Some(3));
  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_remove_after_invalidate() {
  let mut cache = LruCache::new();
  cache.add(1);
  cache.add(2);
  cache.add(3);

  cache.invalidate_one(); // Removes 1
  cache.remove(&2);

  // Should only have 3
  assert_eq!(cache.invalidate_one(), Some(3));
  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_large_number_of_items() {
  let mut cache = LruCache::with_capacity(1000);

  for i in 0..1000 {
    cache.add(i);
  }

  // Invalidate first 500
  for i in 0..500 {
    assert_eq!(cache.invalidate_one(), Some(i));
  }

  // Add 500 more
  for i in 1000..1500 {
    cache.add(i);
  }

  // Should have items 500..1500
  for i in 500..1000 {
    assert_eq!(cache.invalidate_one(), Some(i));
  }

  for i in 1000..1500 {
    assert_eq!(cache.invalidate_one(), Some(i));
  }

  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_interleaved_operations() {
  let mut cache = LruCache::new();

  cache.add(1);
  cache.add(2);
  assert_eq!(cache.invalidate_one(), Some(1));
  cache.add(3);
  cache.remove(&2);
  cache.add(4);
  assert_eq!(cache.invalidate_one(), Some(3));
  cache.add(5);
  assert_eq!(cache.invalidate_one(), Some(4));
  assert_eq!(cache.invalidate_one(), Some(5));
  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_with_complex_types() {
  #[derive(Clone, Hash, Eq, PartialEq, Debug)]
  struct ComplexType {
    id: u32,
    name: String,
  }

  let mut cache = LruCache::new();

  cache.add(ComplexType {
    id: 1,
    name: "first".to_string(),
  });

  cache.add(ComplexType {
    id: 2,
    name: "second".to_string(),
  });

  let removed = cache.invalidate_one();
  assert!(removed.is_some());
  assert_eq!(removed.unwrap().id, 1);
}

#[test]
fn test_add_remove_same_value_multiple_times() {
  let mut cache = LruCache::new();

  // Add and remove the same value multiple times
  for _ in 0..10 {
    cache.add(42);
    cache.remove(&42);
  }

  assert_eq!(cache.invalidate_one(), None);
}

#[test]
fn test_duplicate_values_with_different_queue_numbers() {
  let mut cache = LruCache::new();

  // Add the same value multiple times
  cache.add(100);
  cache.add(100);
  cache.add(100);

  // Each has different queue number, so invalidating one
  // will only remove one instance
  assert_eq!(cache.invalidate_one(), Some(100));

  // Trying to remove 100 should remove one from the map
  // (whichever one the HashMap decides)
  cache.remove(&100);

  // Depending on which one was removed, we may still have items
  let _ = cache.invalidate_one();
}

#[test]
fn test_zero_sized_type() {
  #[derive(Clone, Debug, Hash, Eq, PartialEq)]
  struct ZeroSized;

  let mut cache = LruCache::new();
  cache.add(ZeroSized);
  cache.add(ZeroSized);

  assert_eq!(cache.invalidate_one(), Some(ZeroSized));
  cache.remove(&ZeroSized);
}

#[test]
fn test_add_invalidate_cycle() {
  let mut cache = LruCache::new();

  for cycle in 0..100 {
    cache.add(cycle);
    if cycle % 2 == 0 {
      cache.invalidate_one();
    }
  }

  // Clean up remaining items
  while cache.invalidate_one().is_some() {}
}

#[test]
fn test_remove_with_strings() {
  let mut cache = LruCache::new();

  cache.add("apple".to_string());
  cache.add("banana".to_string());
  cache.add("cherry".to_string());

  cache.remove(&"banana".to_string());

  let first = cache.invalidate_one().unwrap();
  let second = cache.invalidate_one().unwrap();

  // Should have apple and cherry (in some order)
  assert!((first == "apple" && second == "cherry") || (first == "cherry" && second == "apple"));

  assert_eq!(cache.invalidate_one(), None);
}
