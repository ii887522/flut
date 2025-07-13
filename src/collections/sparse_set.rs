use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
  cell::UnsafeCell,
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicU32, Ordering},
};

struct DenseItem<T> {
  id: u32,
  value: T,
}

struct DenseCell<T>(UnsafeCell<DenseItem<T>>);

unsafe impl<T> Sync for DenseCell<T> {}

impl<T> DenseCell<T> {
  const fn new(id: u32, item: T) -> Self {
    Self(UnsafeCell::new(DenseItem { id, value: item }))
  }
}

impl<T> Deref for DenseCell<T> {
  type Target = UnsafeCell<DenseItem<T>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> DerefMut for DenseCell<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub(crate) struct UpdateResp {
  pub(crate) id: u32,
  pub(crate) dense_index: u32,
}

pub(crate) struct RemoveResp<T> {
  pub(crate) id: u32,
  pub(crate) dense_index: u32,
  pub(crate) item: T,
}

pub(crate) struct SparseSet<T> {
  dense: Vec<DenseCell<T>>,
  sparse: Vec<AtomicU32>,
  free_ids: Vec<u32>,
}

impl<T> SparseSet<T> {
  pub(crate) fn with_capacity(capacity: usize) -> Self {
    Self {
      dense: Vec::with_capacity(capacity),
      sparse: Vec::with_capacity(capacity),
      free_ids: Vec::with_capacity(capacity),
    }
  }

  pub(crate) const fn len(&self) -> usize {
    self.dense.len()
  }

  pub(crate) fn push(&mut self, item: T) -> u32 {
    let dense_item_count = self.dense.len();

    let id = if let Some(id) = self.free_ids.pop() {
      *self.sparse[id as usize].get_mut() = dense_item_count as _;
      id
    } else {
      let id = self.sparse.len();
      self.sparse.push(AtomicU32::new(dense_item_count as _));
      id as _
    };

    self.dense.push(DenseCell::new(id, item));
    id
  }

  pub(crate) fn update(&mut self, id: u32, item: T) -> UpdateResp {
    let dense_index = *self.sparse[id as usize].get_mut();
    self.dense[dense_index as usize].get_mut().value = item;
    UpdateResp { id, dense_index }
  }

  pub(crate) fn remove(&mut self, id: u32) -> RemoveResp<T> {
    let dense_index = *self.sparse[id as usize].get_mut();

    let item = self
      .dense
      .swap_remove(dense_index as _)
      .0
      .into_inner()
      .value;

    if dense_index < self.dense.len() as _ {
      *self.sparse[self.dense[dense_index as usize].get_mut().id as usize].get_mut() =
        dense_index as _;
    }

    self.free_ids.push(id);

    RemoveResp {
      id,
      dense_index,
      item,
    }
  }
}

impl<T: Send> SparseSet<T> {
  pub(crate) fn par_extend(&mut self, items: Vec<T>) -> Box<[u32]> {
    let item_count = items.len();
    let dense_item_count = self.dense.len();
    let free_id_count = self.free_ids.len();

    let mut ids = self
      .free_ids
      .par_drain(free_id_count.saturating_sub(item_count)..)
      .enumerate()
      .map(|(index, id)| {
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

    self.dense.par_extend(
      ids
        .par_iter()
        .zip(items.into_par_iter())
        .map(|(&id, item)| DenseCell::new(id, item)),
    );

    ids.into_boxed_slice()
  }

  pub(crate) fn par_update(&mut self, items: FxHashMap<u32, T>) -> Box<[UpdateResp]> {
    items
      .into_par_iter()
      .map(|(id, item)| unsafe {
        let dense_index = self.sparse[id as usize].load(Ordering::Relaxed);
        (*self.dense[dense_index as usize].get()).value = item;
        UpdateResp { id, dense_index }
      })
      .collect()
  }
}

impl<T: Send + Copy> SparseSet<T> {
  pub(crate) fn par_remove(&mut self, ids: FxHashSet<u32>) -> Box<[RemoveResp<T>]> {
    let dense_indices = ids
      .par_iter()
      .map(|&id| self.sparse[id as usize].load(Ordering::Relaxed))
      .collect::<FxHashSet<_>>();

    let resps = dense_indices
      .par_iter()
      .map(|&dense_index| unsafe {
        let dense_item = self.dense[dense_index as usize].get();

        RemoveResp {
          id: (*dense_item).id,
          dense_index,
          item: (*dense_item).value,
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

    let dst_dense_indices = dense_indices
      .into_par_iter()
      .filter(|&dense_index| dense_index < dense_start_index as _)
      .collect::<Box<_>>();

    src_dense_cells
      .into_par_iter()
      .zip(dst_dense_indices.into_par_iter())
      .for_each(|(dense_cell, &dense_index)| unsafe {
        let dst_item = self.dense[dense_index as usize].get();
        let src_item = dense_cell.get();
        (*dst_item).id = (*src_item).id;
        (*dst_item).value = (*src_item).value;
        self.sparse[(*dst_item).id as usize].store(dense_index, Ordering::Relaxed);
      });

    self.dense.truncate(dense_start_index);
    self.free_ids.par_extend(ids.into_par_iter());
    resps
  }
}
