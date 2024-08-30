use rand::prelude::*;
use rayon::prelude::*;

#[derive(Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct U16SparseSet {
  dense: Vec<u16>,
  sparse: Vec<u16>,
}

impl U16SparseSet {
  pub fn from_par_iter(iter: impl IntoParallelIterator<Item = u16>) -> Self {
    let dense = iter.into_par_iter().collect::<Vec<_>>();
    let mut sparse = vec![0; dense.len()];

    for (dense_index, &sparse_index) in dense.iter().enumerate() {
      if sparse_index >= sparse.len() as _ {
        sparse.resize((sparse_index + 1) as _, 0);
      }

      sparse[sparse_index as usize] = dense_index as _;
    }

    Self { dense, sparse }
  }

  pub fn len(&self) -> u16 {
    self.dense.len() as _
  }

  pub fn is_empty(&self) -> bool {
    self.dense.is_empty()
  }

  pub fn swap_remove(&mut self, value: u16) {
    let dense_index: u16 = self.sparse[value as usize];

    if self.dense[dense_index as usize] != value {
      panic!("{value} does not exist")
    }

    self.dense.swap_remove(dense_index as _);

    if let Some(&sparse_index) = self.dense.get(dense_index as usize) {
      self.sparse[sparse_index as usize] = dense_index;
    }
  }

  pub fn random(&self) -> Option<u16> {
    self.dense.choose(&mut thread_rng()).copied()
  }

  pub fn push(&mut self, value: u16) {
    if value >= self.sparse.len() as _ {
      self.sparse.resize((value + 1) as _, 0);
    }

    self.sparse[value as usize] = self.dense.len() as _;
    self.dense.push(value);
  }
}
