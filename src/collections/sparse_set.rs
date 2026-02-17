use rustc_hash::FxHashSet;
use std::{cmp::Ordering, mem};
use voracious_radix_sort::{RadixSort as _, Radixable};

pub struct AddResp {
  pub id: u32,
}

pub struct UpdateResp {
  pub index: u32,
}

pub struct RemoveResp<T> {
  pub item: T,
  pub index: Option<u32>,
}

pub struct BulkAddResp {
  pub ids: Box<[u32]>,
}

pub struct BulkUpdateResp {
  pub indices: Box<[u32]>,
}

pub struct BulkRemoveResp<T> {
  pub items: Box<[T]>,
  pub indices: Box<[u32]>,
}

pub struct SortResp {
  pub indices: Box<[u32]>,
}

#[derive(Clone, Copy)]
struct ItemMeta {
  id: u32,
  seq: u32,
}

impl PartialEq for ItemMeta {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.seq == other.seq
  }
}

impl PartialOrd for ItemMeta {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.seq.partial_cmp(&other.seq)
  }
}

impl Radixable<u32> for ItemMeta {
  type Key = u32;

  #[inline]
  fn key(&self) -> Self::Key {
    self.seq
  }
}

#[derive(Clone, Copy)]
struct ItemWithMeta<T> {
  item: T,
  meta: ItemMeta,
}

impl<T: PartialEq> PartialEq for ItemWithMeta<T> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.item == other.item && self.meta == other.meta
  }
}

impl<T: PartialOrd> PartialOrd for ItemWithMeta<T> {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match self.item.partial_cmp(&other.item) {
      Some(Ordering::Equal) => (),
      ord => return ord,
    }

    self.meta.partial_cmp(&other.meta)
  }
}

impl<T: Radixable<f32, Key = f32>> Radixable<u64> for ItemWithMeta<T> {
  type Key = u64;

  #[inline]
  fn key(&self) -> Self::Key {
    (u64::from(self.item.key().to_bits()) << 32) | (u64::from(self.meta.key()))
  }
}

#[must_use]
pub struct SparseSet<T> {
  items: Vec<T>,
  item_metas: Vec<ItemMeta>,
  id_to_index: Vec<u32>,
  free_ids: Vec<u32>,
  next_seq: u32,
}

impl<T> Default for SparseSet<T> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<T> SparseSet<T> {
  #[inline]
  pub const fn new() -> Self {
    Self {
      items: vec![],
      item_metas: vec![],
      id_to_index: vec![],
      free_ids: vec![],
      next_seq: 0,
    }
  }

  #[inline]
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      items: Vec::with_capacity(capacity),
      item_metas: Vec::with_capacity(capacity),
      id_to_index: Vec::with_capacity(capacity),
      free_ids: Vec::with_capacity(capacity),
      next_seq: 0,
    }
  }

  #[must_use]
  #[inline]
  pub const fn is_empty(&self) -> bool {
    self.items.is_empty()
  }

  #[must_use]
  #[inline]
  pub const fn len(&self) -> usize {
    self.items.len()
  }

  #[must_use]
  #[inline]
  pub fn get_items(&self) -> &[T] {
    &self.items
  }

  pub fn add(&mut self, item: T) -> AddResp {
    let item_count = self.items.len().try_into().unwrap();

    let id = if let Some(id) = self.free_ids.pop() {
      self.id_to_index[id as usize] = item_count;
      id
    } else {
      let id = self.id_to_index.len().try_into().unwrap();
      self.id_to_index.push(item_count);
      id
    };

    self.items.push(item);

    self.item_metas.push(ItemMeta {
      id,
      seq: self.next_seq,
    });

    self.next_seq += 1;
    AddResp { id }
  }

  pub fn update(&mut self, id: u32, item: T) -> UpdateResp {
    let index = self.id_to_index[id as usize];
    self.items[index as usize] = item;
    UpdateResp { index }
  }

  pub fn remove(&mut self, id: u32) -> RemoveResp<T> {
    let index = self.id_to_index[id as usize];
    let item = self.items.swap_remove(index as usize);
    self.item_metas.swap_remove(index as usize);

    let index = if let Some(&ItemMeta {
      id: moved_id,
      seq: _,
    }) = self.item_metas.get(index as usize)
    {
      self.id_to_index[moved_id as usize] = index;
      Some(index)
    } else {
      None
    };

    self.free_ids.push(id);
    RemoveResp { item, index }
  }

  pub fn bulk_add(&mut self, items: Box<[T]>) -> BulkAddResp {
    let free_id_count = self.free_ids.len();
    let id_to_index_count = self.id_to_index.len();

    let mut ids = self
      .free_ids
      .drain(free_id_count.saturating_sub(items.len())..)
      .enumerate()
      .map(|(index, id)| {
        self.id_to_index[id as usize] = (self.items.len() + index).try_into().unwrap();
        id
      })
      .collect::<Vec<_>>();

    self.id_to_index.extend(
      (0..items.len().saturating_sub(free_id_count))
        .map(|index| u32::try_from(self.items.len() + ids.len() + index).unwrap()),
    );

    ids.extend(
      u32::try_from(id_to_index_count).unwrap()
        ..u32::try_from(id_to_index_count + items.len() - free_id_count).unwrap(),
    );

    let ids = ids;
    self.items.extend(items);

    self
      .item_metas
      .extend(ids.iter().enumerate().map(|(index, &id)| ItemMeta {
        id,
        seq: self.next_seq + index as u32,
      }));

    self.next_seq += ids.len() as u32;

    BulkAddResp {
      ids: ids.into_boxed_slice(),
    }
  }

  pub fn bulk_update(&mut self, ids: &[u32], items: Box<[T]>) -> BulkUpdateResp {
    let indices = ids
      .iter()
      .zip(items)
      .map(|(&id, item)| {
        let index = self.id_to_index[id as usize];
        self.items[index as usize] = item;
        index
      })
      .collect();

    BulkUpdateResp { indices }
  }
}

impl<T: Clone> SparseSet<T> {
  pub fn bulk_remove(&mut self, ids: &[u32]) -> BulkRemoveResp<T> {
    let remaining_item_count = self.items.len().saturating_sub(ids.len());

    let indices = ids
      .iter()
      .map(|&id| self.id_to_index[id as usize])
      .collect::<Box<_>>();

    let items = indices
      .iter()
      .map(|&index| self.items[index as usize].clone())
      .collect::<Box<_>>();

    let indices_to_replace = indices
      .iter()
      .copied()
      .filter(|&index| (index as usize) < remaining_item_count)
      .collect::<Box<_>>();

    let indices = FxHashSet::from_iter(indices);

    let items_to_move = self
      .items
      .drain(remaining_item_count..)
      .zip(self.item_metas.drain(remaining_item_count..))
      .enumerate()
      .filter_map(|(index, (item, item_meta))| {
        let index = (remaining_item_count + index).try_into().unwrap();

        if indices.contains(&index) {
          None
        } else {
          Some((item_meta, item))
        }
      })
      .collect::<Box<_>>();

    indices_to_replace.iter().zip(items_to_move).for_each(
      |(&index_to_replace, (item_meta_to_move, item_to_move))| {
        self.id_to_index[item_meta_to_move.id as usize] = index_to_replace;
        self.items[index_to_replace as usize] = item_to_move;
        self.item_metas[index_to_replace as usize] = item_meta_to_move;
      },
    );

    self.free_ids.extend(ids);

    BulkRemoveResp {
      items,
      indices: indices_to_replace,
    }
  }
}

impl<T: Radixable<f32, Key = f32>> SparseSet<T> {
  pub fn sort(&mut self) -> SortResp {
    let mut item_with_metas = mem::take(&mut self.items)
      .into_iter()
      .zip(mem::take(&mut self.item_metas))
      .map(|(item, item_meta)| ItemWithMeta {
        item,
        meta: item_meta,
      })
      .collect::<Box<_>>();

    item_with_metas.voracious_sort();
    let mut indices = Vec::with_capacity(item_with_metas.len());

    let (items, item_metas): (Vec<_>, Vec<_>) = item_with_metas
      .into_iter()
      .enumerate()
      .map(
        |(
          index,
          ItemWithMeta {
            item,
            meta: ItemMeta { id, seq: _ },
          },
        )| {
          let old_index = self.id_to_index[id as usize];
          let index = index as u32;
          self.id_to_index[id as usize] = index;

          if index != old_index {
            indices.push(index);
          }

          (item, ItemMeta { id, seq: index })
        },
      )
      .unzip();

    self.next_seq = items.len() as u32;
    self.items = items;
    self.item_metas = item_metas;

    SortResp {
      indices: indices.into_boxed_slice(),
    }
  }
}
