use std::ops::{Index, IndexMut};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SparseVec<T> {
  anys: Vec<Option<T>>,
  free_indices: Vec<u32>,
}

impl<T> Default for SparseVec<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> From<T> for SparseVec<T> {
  fn from(any: T) -> Self {
    Self {
      anys: vec![Some(any)],
      free_indices: vec![],
    }
  }
}

impl<T> SparseVec<T> {
  pub const fn new() -> Self {
    Self {
      anys: vec![],
      free_indices: vec![],
    }
  }

  pub fn push(&mut self, any: T) -> u32 {
    if let Some(free_index) = self.free_indices.pop() {
      self.anys[free_index as usize] = Some(any);
      free_index
    } else {
      let free_index = self.anys.len();
      self.anys.push(Some(any));
      free_index as _
    }
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn len(&self) -> u32 {
    (self.anys.len() - self.free_indices.len()) as _
  }

  pub fn replace_with(&mut self, index: u32, f: impl FnOnce(T) -> Option<T>) {
    let any = &mut self.anys[index as usize];

    if let Some(replacement) = f(any.take().unwrap()) {
      *any = Some(replacement);
    } else {
      self.free_indices.push(index);
    }
  }

  pub fn take(&mut self, index: u32) -> T {
    let any = self.anys[index as usize].take().unwrap();
    self.free_indices.push(index);
    any
  }
}

impl<T> Index<u32> for SparseVec<T> {
  type Output = T;

  fn index(&self, index: u32) -> &Self::Output {
    self.anys[index as usize].as_ref().unwrap()
  }
}

impl<T> IndexMut<u32> for SparseVec<T> {
  fn index_mut(&mut self, index: u32) -> &mut Self::Output {
    self.anys[index as usize].as_mut().unwrap()
  }
}
