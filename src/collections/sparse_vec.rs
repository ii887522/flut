use std::{
  collections::VecDeque,
  ops::{Index, IndexMut},
};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SparseVec<T> {
  anys: Vec<Option<T>>,
  free_indices: VecDeque<u32>,
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
      free_indices: VecDeque::new(),
    }
  }
}

impl<T> SparseVec<T> {
  pub const fn new() -> Self {
    Self {
      anys: vec![],
      free_indices: VecDeque::new(),
    }
  }

  pub fn push(&mut self, any: T) -> u32 {
    if let Some(free_index) = self.free_indices.pop_front() {
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

  pub fn replace_with_and_return<R>(
    &mut self,
    index: u32,
    f: impl FnOnce(T) -> (Option<T>, R),
  ) -> R {
    let any = &mut self.anys[index as usize];
    let (replacement, resp) = f(any.take().unwrap());

    if let Some(replacement) = replacement {
      *any = Some(replacement);
    } else {
      self.free_indices.push_back(index);
    }

    resp
  }

  pub fn take(&mut self, index: u32) -> T {
    let any = self.anys[index as usize].take().unwrap();
    self.free_indices.push_back(index);
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
