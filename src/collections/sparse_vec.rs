use atomic_refcell::AtomicRefCell;
use rayon::prelude::*;
use std::{
  collections::HashSet,
  ops::{Index, IndexMut},
  sync::atomic::{AtomicU16, Ordering},
};

pub struct SparseVec<T> {
  dense: Vec<(AtomicU16, AtomicRefCell<T>)>, // Mappings from index to (id, element)
  sparse: Vec<AtomicU16>,                    // Mappings from id to index
  free_ids: Vec<u16>,                        // Elements removed by their ids
}

impl<T> Default for SparseVec<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> SparseVec<T> {
  pub const fn new() -> Self {
    Self {
      dense: vec![],
      sparse: vec![],
      free_ids: vec![],
    }
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      dense: Vec::with_capacity(capacity),
      sparse: Vec::with_capacity(capacity),
      free_ids: Vec::with_capacity(capacity),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn len(&self) -> usize {
    self.dense.len()
  }

  pub fn get_dense_index(&self, id: u16) -> u16 {
    self.sparse[id as usize].load(Ordering::Relaxed)
  }

  pub fn get_by_dense_index(&self, index: u16) -> Option<&AtomicRefCell<T>> {
    self.dense.get(index as usize).map(|(_, value)| value)
  }

  pub fn get_dense(&self) -> &[(AtomicU16, AtomicRefCell<T>)] {
    &self.dense
  }

  pub fn push(&mut self, value: T) -> u16 {
    let dense_index = self.dense.len();

    let id = if let Some(id) = self.free_ids.pop() {
      self.sparse[id as usize] = AtomicU16::new(dense_index as _);
      id
    } else {
      let id = self.sparse.len();
      self.sparse.push(AtomicU16::new(dense_index as _));
      id as _
    };

    self
      .dense
      .push((AtomicU16::new(id), AtomicRefCell::new(value)));

    id
  }

  pub fn remove(&mut self, id: u16) -> T {
    let dense_index = self.sparse[id as usize].swap(u16::MAX, Ordering::Relaxed);
    let (_, result) = self.dense.swap_remove(dense_index as _);

    if let Some((moved_id, _)) = self.dense.get_mut(dense_index as usize) {
      self.sparse[*moved_id.get_mut() as usize] = AtomicU16::new(dense_index);
    }

    self.free_ids.push(id);
    result.into_inner()
  }

  pub fn clear(&mut self) {
    self.dense.clear();
    self.sparse.clear();
    self.free_ids.clear();
  }

  pub fn remove_by_dense_index(&mut self, index: u16) -> T {
    let (id, _) = &mut self.dense[index as usize];
    let id = *id.get_mut();
    self.remove(id)
  }
}

impl<T: Send + Sync> SparseVec<T> {
  pub fn par_extend(&mut self, values: Vec<T>) -> Vec<u16> {
    let free_id_count = self.free_ids.len();
    let value_count = values.len();
    let remaining_count = value_count.saturating_sub(free_id_count);
    let dense_start_index = self.dense.len();

    let mut ids = self
      .free_ids
      .par_drain(free_id_count.saturating_sub(value_count)..)
      .collect::<Vec<_>>();

    ids.par_iter().enumerate().for_each(|(index, &id)| {
      self.sparse[id as usize].store((dense_start_index + index) as _, Ordering::Relaxed);
    });

    let start_id = self.sparse.len();

    self.sparse.par_extend(
      (0..remaining_count)
        .into_par_iter()
        .map(|index| AtomicU16::new((dense_start_index + ids.len() + index) as _)),
    );

    ids.par_extend(start_id as u16..(start_id + remaining_count) as _);

    self.dense.par_extend(
      ids
        .par_iter()
        .zip(values.into_par_iter())
        .map(|(&id, value)| (AtomicU16::new(id), AtomicRefCell::new(value))),
    );

    ids
  }

  pub fn par_set(&mut self, ids: &[u16], values: Vec<T>) {
    ids
      .par_iter()
      .zip(values.into_par_iter())
      .for_each(|(&id, value)| *self[id].borrow_mut() = value);
  }
}

impl<T: Send + Sync + Copy> SparseVec<T> {
  pub fn par_remove(&mut self, ids: &[u16]) {
    let dense_indices = ids
      .par_iter()
      .map(|&id| self.sparse[id as usize].swap(u16::MAX, Ordering::Relaxed))
      .collect::<HashSet<_>>();

    let dense_indices_to_move_from = (self.dense.len() - ids.len()..self.dense.len())
      .into_par_iter()
      .filter(|&index| !dense_indices.contains(&(index as _)))
      .collect::<Vec<_>>();

    let dense_indices_to_move_to = dense_indices
      .into_par_iter()
      .filter(|&index| (index as usize) < self.dense.len() - ids.len())
      .collect::<Vec<_>>();

    dense_indices_to_move_from
      .into_par_iter()
      .zip(dense_indices_to_move_to.into_par_iter())
      .for_each(|(from_index, to_index)| {
        let (from_id, from_value) = &self.dense[from_index];
        let (to_id, to_value) = &self.dense[to_index as usize];
        let from_id = from_id.load(Ordering::Relaxed);
        to_id.store(from_id, Ordering::Relaxed);
        *to_value.borrow_mut() = *from_value.borrow();
        self.sparse[from_id as usize].store(to_index, Ordering::Relaxed);
      });

    self.dense.truncate(self.dense.len() - ids.len());
    self.free_ids.par_extend(ids);
  }
}

impl<T> Index<u16> for SparseVec<T> {
  type Output = AtomicRefCell<T>;

  fn index(&self, id: u16) -> &Self::Output {
    let dense_index = self.get_dense_index(id);
    &self.dense[dense_index as usize].1
  }
}

impl<T> IndexMut<u16> for SparseVec<T> {
  fn index_mut(&mut self, id: u16) -> &mut Self::Output {
    let dense_index = self.get_dense_index(id);
    &mut self.dense[dense_index as usize].1
  }
}
