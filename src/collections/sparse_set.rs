use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
  cell::UnsafeCell,
  collections::BTreeSet,
  ops::{Deref, DerefMut},
  slice,
  sync::atomic::{AtomicU32, Ordering},
};

struct DenseCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for DenseCell<T> {}

impl<T> DenseCell<T> {
  const fn new(item: T) -> Self {
    Self(UnsafeCell::new(item))
  }
}

impl<T> Deref for DenseCell<T> {
  type Target = UnsafeCell<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for DenseCell<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub struct UpdateResp {
  pub id: u32,
  pub dense_index: u32,
}

pub struct RemoveResp<T> {
  pub id: u32,
  pub dense_index: u32,
  pub item: T,
}

pub struct SparseSet<T> {
  dense: Vec<DenseCell<T>>,
  dense_ids: Vec<AtomicU32>,
  sparse: Vec<AtomicU32>,
  free_ids: BTreeSet<u32>,
}

impl<T> SparseSet<T> {
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      dense: Vec::with_capacity(capacity),
      dense_ids: Vec::with_capacity(capacity),
      sparse: Vec::with_capacity(capacity),
      free_ids: BTreeSet::new(),
    }
  }

  pub const fn len(&self) -> usize {
    self.dense.len()
  }

  pub const fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub const fn get_dense(&self) -> &[T] {
    unsafe { slice::from_raw_parts(self.dense.as_ptr() as *const _, self.dense.len()) }
  }

  pub const fn get_dense_mut(&mut self) -> &mut [T] {
    unsafe { slice::from_raw_parts_mut(self.dense.as_mut_ptr() as *mut _, self.dense.len()) }
  }

  pub(crate) fn get_dense_ptr(&self) -> *const T {
    self.dense.as_ptr() as *const _
  }

  pub const fn get_dense_ids(&self) -> &[u32] {
    unsafe { slice::from_raw_parts(self.dense_ids.as_ptr() as *const _, self.dense_ids.len()) }
  }

  pub fn push(&mut self, item: T) -> u32 {
    let dense_item_count = self.dense.len();

    let id = if let Some(id) = self.free_ids.pop_first() {
      *self.sparse[id as usize].get_mut() = dense_item_count as _;
      id
    } else {
      let id = self.sparse.len();
      self.sparse.push(AtomicU32::new(dense_item_count as _));
      id as _
    };

    self.dense.push(DenseCell::new(item));
    self.dense_ids.push(AtomicU32::new(id));
    id
  }

  pub fn push_by_id(&mut self, id: u32, item: T) {
    if !self.free_ids.remove(&id) {
      return;
    }

    *self.sparse[id as usize].get_mut() = self.dense.len() as _;
    self.dense.push(DenseCell::new(item));
    self.dense_ids.push(AtomicU32::new(id));
  }

  pub fn update(&mut self, id: u32, item: T) -> UpdateResp {
    let dense_index = *self.sparse[id as usize].get_mut();
    *self.dense[dense_index as usize].get_mut() = item;
    UpdateResp { id, dense_index }
  }

  pub fn remove(&mut self, id: u32) -> Option<RemoveResp<T>> {
    if id >= self.sparse.len() as _ || !self.free_ids.insert(id) {
      return None;
    }

    let dense_index = *self.sparse[id as usize].get_mut();
    let item = self.dense.swap_remove(dense_index as _).0.into_inner();
    let id = *self.dense_ids.swap_remove(dense_index as _).get_mut();

    if dense_index < self.dense.len() as _ {
      *self.sparse[*self.dense_ids[dense_index as usize].get_mut() as usize].get_mut() =
        dense_index as _;
    }

    Some(RemoveResp {
      id,
      dense_index,
      item,
    })
  }
}

impl<T: Send> SparseSet<T> {
  pub fn from_par_iter(par_iter: impl IntoParallelIterator<Item = (u32, T)>) -> Self {
    let (dense_ids, dense): (Vec<_>, Vec<_>) = par_iter
      .into_par_iter()
      .map(|(id, item)| (AtomicU32::new(id), DenseCell::new(item)))
      .unzip();

    let max_id = dense_ids
      .par_iter()
      .max_by_key(|&id| id.load(Ordering::Relaxed))
      .unwrap()
      .load(Ordering::Relaxed);

    let dense_id_set =
      FxHashSet::from_par_iter(dense_ids.par_iter().map(|id| id.load(Ordering::Relaxed)));

    let free_ids = (0..=max_id)
      .into_par_iter()
      .filter(|id| !dense_id_set.contains(id))
      .collect();

    let sparse = Vec::from_par_iter(
      (0..=max_id)
        .into_par_iter()
        .map(|_| AtomicU32::new(u32::MAX)),
    );

    dense_ids
      .par_iter()
      .enumerate()
      .for_each(|(index, dense_id)| {
        sparse[dense_id.load(Ordering::Relaxed) as usize].store(index as _, Ordering::Relaxed);
      });

    Self {
      dense,
      dense_ids,
      sparse,
      free_ids,
    }
  }

  pub fn par_extend(&mut self, items: Vec<T>) -> Box<[u32]> {
    let item_count = items.len();
    let dense_item_count = self.dense.len();
    let free_id_count = self.free_ids.len();

    let mut ids = (0..item_count.min(free_id_count))
      .map(|index| {
        let id = self.free_ids.pop_first().unwrap();
        self.sparse[id as usize].store((dense_item_count + index) as _, Ordering::Relaxed);
        id
      })
      .collect::<Vec<_>>();

    let sparse_index_count = self.sparse.len();
    let remaining_item_count = item_count.saturating_sub(free_id_count);

    self.sparse.par_extend(
      (0..remaining_item_count)
        .into_par_iter()
        .map(|index| AtomicU32::new((dense_item_count + free_id_count + index) as _)),
    );

    ids.par_extend(
      (0..remaining_item_count)
        .into_par_iter()
        .map(|index| (sparse_index_count + index) as u32),
    );

    self
      .dense
      .par_extend(items.into_par_iter().map(|item| DenseCell::new(item)));

    self
      .dense_ids
      .par_extend(ids.par_iter().map(|&id| AtomicU32::new(id)));

    ids.into_boxed_slice()
  }

  pub fn par_update(&mut self, items: FxHashMap<u32, T>) -> Box<[UpdateResp]> {
    items
      .into_par_iter()
      .map(|(id, item)| unsafe {
        let dense_index = self.sparse[id as usize].load(Ordering::Relaxed);
        *self.dense[dense_index as usize].get() = item;
        UpdateResp { id, dense_index }
      })
      .collect()
  }
}

impl<T: Send + Copy> SparseSet<T> {
  pub fn par_remove(&mut self, ids: FxHashSet<u32>) -> Box<[RemoveResp<T>]> {
    let ids = ids
      .into_par_iter()
      .filter(|&id| id < self.sparse.len() as _ && !self.free_ids.contains(&id))
      .collect::<Vec<_>>();

    let dense_indices = ids
      .par_iter()
      .map(|&id| self.sparse[id as usize].load(Ordering::Relaxed))
      .collect::<FxHashSet<_>>();

    let resps = dense_indices
      .par_iter()
      .map(|&dense_index| unsafe {
        let dense_item = *self.dense[dense_index as usize].get();
        let dense_id = self.dense_ids[dense_index as usize].load(Ordering::Relaxed);

        RemoveResp {
          id: dense_id,
          dense_index,
          item: dense_item,
        }
      })
      .collect::<Box<_>>();

    let dense_start_index = self.dense.len() - ids.len();

    let src_dense_cells = self
      .dense
      .par_drain(dense_start_index..)
      .enumerate()
      .filter_map(|(index, dense_cell)| {
        if dense_indices.contains(&((dense_start_index + index) as _)) {
          None
        } else {
          Some(dense_cell)
        }
      })
      .collect::<Box<_>>();

    let src_dense_ids = self
      .dense_ids
      .par_drain(dense_start_index..)
      .enumerate()
      .filter_map(|(index, dense_id)| {
        if dense_indices.contains(&((dense_start_index + index) as _)) {
          None
        } else {
          Some(dense_id)
        }
      })
      .collect::<Box<_>>();

    let dst_dense_indices = dense_indices
      .into_par_iter()
      .filter(|&dense_index| dense_index < dense_start_index as _)
      .collect::<Box<_>>();

    src_dense_cells
      .into_par_iter()
      .zip(src_dense_ids.into_par_iter())
      .zip(dst_dense_indices.into_par_iter())
      .for_each(
        |((src_dense_cell, src_dense_id), &dst_dense_index)| unsafe {
          *self.dense[dst_dense_index as usize].get() = *src_dense_cell.get();
          let src_dense_id = src_dense_id.load(Ordering::Relaxed);
          self.dense_ids[dst_dense_index as usize].store(src_dense_id, Ordering::Relaxed);
          self.sparse[src_dense_id as usize].store(dst_dense_index, Ordering::Relaxed);
        },
      );

    self.dense.truncate(dense_start_index);
    self.dense_ids.truncate(dense_start_index);
    self.free_ids.par_extend(ids);
    resps
  }
}
