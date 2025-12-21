use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{collections::BTreeMap, hash::Hash};

pub struct LruCache<T> {
  q_num_to_item: BTreeMap<u32, T>,
  item_to_q_num: FxHashMap<T, u32>,
  next_q_num: u32,
}

impl<T> Default for LruCache<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> LruCache<T> {
  #[inline]
  pub fn new() -> Self {
    Self {
      q_num_to_item: BTreeMap::new(),
      item_to_q_num: FxHashMap::default(),
      next_q_num: 0,
    }
  }

  #[inline]
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      q_num_to_item: BTreeMap::new(),
      item_to_q_num: FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher),
      next_q_num: 0,
    }
  }
}

impl<T: Hash + Eq> LruCache<T> {
  pub fn invalidate_one(&mut self) -> Option<T> {
    let (_, item) = self.q_num_to_item.pop_first()?;
    self.item_to_q_num.remove(&item);
    Some(item)
  }

  pub fn remove(&mut self, item: &T) {
    if let Some(q_num) = self.item_to_q_num.remove(item) {
      self.q_num_to_item.remove(&q_num);
    }
  }
}

impl<T: Clone + Hash + Eq> LruCache<T> {
  pub fn add(&mut self, item: T) {
    self.q_num_to_item.insert(self.next_q_num, item.clone());
    self.item_to_q_num.insert(item, self.next_q_num);
    self.next_q_num += 1;
  }
}
