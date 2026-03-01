use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{collections::BTreeMap, hash::Hash};

struct CacheValue<V> {
  value: V,
  seq: u32,
}

#[must_use]
pub struct LruCache<K, V> {
  seq_to_key: BTreeMap<u32, K>,
  key_to_value: FxHashMap<K, CacheValue<V>>,
  next_seq: u32,
}

impl<K, V> Default for LruCache<K, V> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<K, V> LruCache<K, V> {
  #[inline]
  pub fn new() -> Self {
    Self {
      seq_to_key: BTreeMap::new(),
      key_to_value: FxHashMap::default(),
      next_seq: 0,
    }
  }

  #[inline]
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      seq_to_key: BTreeMap::new(),
      key_to_value: FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher),
      next_seq: 0,
    }
  }
}

impl<K: Clone + Eq + Hash, V> LruCache<K, V> {
  pub fn insert(&mut self, key: K, value: V) {
    self.seq_to_key.insert(self.next_seq, key.clone());

    self.key_to_value.insert(
      key,
      CacheValue {
        value,
        seq: self.next_seq,
      },
    );

    self.next_seq += 1;
  }

  pub fn remove(&mut self, key: &K) -> Option<V> {
    self
      .key_to_value
      .remove(key)
      .map(|CacheValue { value, seq }| {
        self.seq_to_key.remove(&seq);
        value
      })
  }
}

impl<K: Eq + Hash, V> LruCache<K, V> {
  pub fn evict_one(&mut self) -> Option<(K, V)> {
    if let Some((_seq, key)) = self.seq_to_key.pop_first() {
      self
        .key_to_value
        .remove(&key)
        .map(|CacheValue { value, seq: _ }| (key, value))
    } else {
      None
    }
  }
}
