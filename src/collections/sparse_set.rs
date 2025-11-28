use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
  slice,
  sync::atomic::{AtomicU32, Ordering},
};

const MIN_SEQ_LEN: usize = 1024;

#[derive(Clone, Copy)]
pub struct Id(u32);

struct DenseCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for DenseCell<T> {}

impl<T> DenseCell<T> {
  #[inline]
  const fn new(value: T) -> Self {
    Self(UnsafeCell::new(value))
  }
}

impl<T> Deref for DenseCell<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.0.get() }
  }
}

impl<T> DerefMut for DenseCell<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.0.get() }
  }
}

pub struct AddResp {
  pub id: Id,
  pub index: u32,
}

pub struct UpdateResp {
  pub index: u32,
}

pub struct RemoveResp {
  pub index: Option<u32>,
}

pub struct BulkAddResp {
  pub ids: Box<[Id]>,
  pub index: u32,
  pub len: u32,
}

pub struct BulkUpdateResp {
  pub indices: Box<[u32]>,
}

pub struct BulkRemoveResp {
  pub indices: Box<[u32]>,
}

pub struct SparseSet<T> {
  dense: Vec<DenseCell<T>>,
  ids: Vec<AtomicU32>,
  sparse: Vec<AtomicU32>,
  free_ids: Vec<u32>,
}

impl<T> Default for SparseSet<T> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<T: Send> FromParallelIterator<T> for SparseSet<T> {
  fn from_par_iter<I>(par_iter: I) -> Self
  where
    I: IntoParallelIterator<Item = T>,
  {
    let mut set = Self::new();
    set.bulk_add(par_iter.into_par_iter().collect());
    set
  }
}

impl<T> SparseSet<T> {
  #[inline]
  pub const fn new() -> Self {
    Self {
      dense: vec![],
      ids: vec![],
      sparse: vec![],
      free_ids: vec![],
    }
  }

  #[inline]
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      dense: Vec::with_capacity(capacity),
      ids: Vec::with_capacity(capacity),
      sparse: Vec::with_capacity(capacity),
      free_ids: Vec::with_capacity(capacity),
    }
  }

  #[inline]
  pub const fn is_empty(&self) -> bool {
    self.len() == 0
  }

  #[inline]
  pub const fn len(&self) -> usize {
    self.dense.len()
  }

  #[inline]
  pub const fn get_items(&self) -> &[T] {
    unsafe { slice::from_raw_parts(self.dense.as_ptr() as *const _, self.dense.len()) }
  }

  pub fn add(&mut self, item: T) -> AddResp {
    let index = self.dense.len() as _;
    self.dense.push(DenseCell::new(item));

    let id = if let Some(free_id) = self.free_ids.pop() {
      self.sparse[free_id as usize] = AtomicU32::new(index);
      free_id
    } else {
      let id = self.sparse.len() as _;
      self.sparse.push(AtomicU32::new(index));
      id
    };

    self.ids.push(AtomicU32::new(id));
    AddResp { id: Id(id), index }
  }

  pub fn update(&mut self, id: Id, item: T) -> UpdateResp {
    let index = *self.sparse[id.0 as usize].get_mut();
    self.dense[index as usize] = DenseCell::new(item);
    UpdateResp { index }
  }

  pub fn remove(&mut self, id: Id) -> RemoveResp {
    let dense_index = *self.sparse[id.0 as usize].get_mut();
    self.dense.swap_remove(dense_index as _);
    self.ids.swap_remove(dense_index as _);

    let resp = if let Some(swapped_id) = self.ids.get_mut(dense_index as usize) {
      self.sparse[*swapped_id.get_mut() as usize] = AtomicU32::new(dense_index);

      RemoveResp {
        index: Some(dense_index),
      }
    } else {
      RemoveResp { index: None }
    };

    self.free_ids.push(id.0);
    resp
  }
}

impl<T: Send> SparseSet<T> {
  pub fn bulk_add(&mut self, items: Box<[T]>) -> BulkAddResp {
    let count = items.len();
    let start_index = self.dense.len();

    self.dense.par_extend(
      items
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(DenseCell::new),
    );

    let mut ids = Vec::with_capacity(count);
    let free_id_count = count.min(self.free_ids.len());

    ids.par_extend(
      self
        .free_ids
        .par_drain(self.free_ids.len() - free_id_count..)
        .with_min_len(MIN_SEQ_LEN),
    );

    ids
      .par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .enumerate()
      .for_each(|(index, &id)| {
        self.sparse[id as usize].store((start_index + index) as _, Ordering::Relaxed);
      });

    let new_start_id = self.sparse.len() as u32;
    let new_start_index = start_index + ids.len();
    let new_ids_count = count - free_id_count;

    self.sparse.par_extend(
      (new_start_index..new_start_index + new_ids_count)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(|index| AtomicU32::new(index as _)),
    );

    ids.par_extend(
      (new_start_id..new_start_id + new_ids_count as u32)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN),
    );

    self.ids.par_extend(
      ids
        .par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(|&id| AtomicU32::new(id)),
    );

    let ids = ids
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(Id)
      .collect();

    BulkAddResp {
      ids,
      index: start_index as _,
      len: count as _,
    }
  }

  pub fn bulk_update(&mut self, updates: Box<[(Id, T)]>) -> BulkUpdateResp {
    let indices = updates
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|(id, item)| unsafe {
        let index = self.sparse[id.0 as usize].load(Ordering::Relaxed);
        *self.dense[index as usize].0.get() = item;
        index
      })
      .collect();

    BulkUpdateResp { indices }
  }
}

impl<T: Clone + Send> SparseSet<T> {
  pub fn bulk_remove(&mut self, ids: &[Id]) -> BulkRemoveResp {
    let start_index = self.dense.len() - ids.len();

    let dense_indices_to_drop = ids
      .par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|id| self.sparse[id.0 as usize].load(Ordering::Relaxed))
      .collect::<FxHashSet<_>>();

    let dense_indices_to_keep = (start_index..self.dense.len())
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .filter(|&index| !dense_indices_to_drop.contains(&(index as _)))
      .collect::<Box<_>>();

    let dense_indices_to_replace = dense_indices_to_drop
      .into_par_iter()
      .filter(|&index| index < start_index as u32)
      .collect::<Box<_>>();

    dense_indices_to_replace
      .par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .zip(
        dense_indices_to_keep
          .into_par_iter()
          .with_min_len(MIN_SEQ_LEN),
      )
      .for_each(|(&index_to_replace, index_to_keep)| unsafe {
        (*self.dense[index_to_replace as usize].0.get()).clone_from(&self.dense[index_to_keep]);
        let swapped_id = self.ids[index_to_keep].load(Ordering::Relaxed);
        self.ids[index_to_replace as usize].store(swapped_id, Ordering::Relaxed);
        self.sparse[swapped_id as usize].store(index_to_replace, Ordering::Relaxed);
      });

    self.dense.truncate(start_index);
    self.ids.truncate(start_index);

    self
      .free_ids
      .par_extend(ids.par_iter().with_min_len(MIN_SEQ_LEN).map(|id| id.0));

    BulkRemoveResp {
      indices: dense_indices_to_replace,
    }
  }
}
