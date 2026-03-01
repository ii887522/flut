use flut::collections::lru_cache::LruCache;

#[test]
fn test_new() {
  let cache: LruCache<i32, i32> = LruCache::new();
  // Since we don't have is_empty or len, we test via evict_one
  let mut cache = cache;
  assert!(cache.evict_one().is_none());
}

#[test]
fn test_default() {
  let cache: LruCache<i32, i32> = LruCache::default();
  let mut cache = cache;
  assert!(cache.evict_one().is_none());
}

#[test]
fn test_insert_and_evict_one() {
  let mut cache = LruCache::new();
  cache.insert(1, 10);
  cache.insert(2, 20);

  // Should evict the first one inserted (FIFO behavior in this simple implementation)
  assert_eq!(cache.evict_one(), Some((1, 10)));
  assert_eq!(cache.evict_one(), Some((2, 20)));
  assert!(cache.evict_one().is_none());
}

#[test]
fn test_remove() {
  let mut cache = LruCache::new();
  cache.insert(1, 10);
  cache.insert(2, 20);

  assert_eq!(cache.remove(&1), Some(10));
  assert_eq!(cache.remove(&1), None);

  // Only 2 should be left
  assert_eq!(cache.evict_one(), Some((2, 20)));
  assert!(cache.evict_one().is_none());
}

#[test]
fn test_insert_overwrite() {
  let mut cache = LruCache::new();
  cache.insert(1, 10);
  cache.insert(1, 20); // Overwrite key 1

  // Currently, the implementation might have a bug where evict_one returns None
  // because the first 'seq' for key 1 is still in seq_to_key but key 1's value
  // in key_to_value was updated with a new 'seq'.

  // First evict_one will try to evict key 1 with seq 0.
  // key_to_value.remove(1) will remove the current value (key 1, seq 1, value 20).
  assert_eq!(cache.evict_one(), Some((1, 20)));

  // Second evict_one will try to evict key 1 with seq 1.
  // key_to_value.remove(1) will return None because it was already removed.
  // So it returns None.
  assert_eq!(cache.evict_one(), None);
}

#[test]
fn test_mixed_ops() {
  let mut cache = LruCache::new();
  cache.insert(1, 10);
  cache.insert(2, 20);
  cache.remove(&1);
  cache.insert(3, 30);

  assert_eq!(cache.evict_one(), Some((2, 20)));
  assert_eq!(cache.evict_one(), Some((3, 30)));
  assert!(cache.evict_one().is_none());
}

#[test]
fn test_with_capacity() {
  let mut cache = LruCache::with_capacity(10);
  cache.insert(1, 10);
  assert_eq!(cache.evict_one(), Some((1, 10)));
}
