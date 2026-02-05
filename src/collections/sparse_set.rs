use rustc_hash::FxHashSet;

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

#[must_use]
pub struct SparseSet<T> {
  items: Vec<T>,
  index_to_id: Vec<u32>,
  id_to_index: Vec<u32>,
  free_ids: Vec<u32>,
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
      index_to_id: vec![],
      id_to_index: vec![],
      free_ids: vec![],
    }
  }

  #[inline]
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      items: Vec::with_capacity(capacity),
      index_to_id: Vec::with_capacity(capacity),
      id_to_index: Vec::with_capacity(capacity),
      free_ids: Vec::with_capacity(capacity),
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
    self.index_to_id.push(id);
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
    self.index_to_id.swap_remove(index as usize);

    let index = if let Some(&moved_id) = self.index_to_id.get(index as usize) {
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
    self.index_to_id.extend(&ids);

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
      .zip(self.index_to_id.drain(remaining_item_count..))
      .enumerate()
      .filter_map(|(index, (item, id))| {
        let index = (remaining_item_count + index).try_into().unwrap();

        if indices.contains(&index) {
          None
        } else {
          Some((id, item))
        }
      })
      .collect::<Box<_>>();

    indices_to_replace.iter().zip(items_to_move).for_each(
      |(&index_to_replace, (id_to_move, item_to_move))| {
        self.items[index_to_replace as usize] = item_to_move;
        self.index_to_id[index_to_replace as usize] = id_to_move;
        self.id_to_index[id_to_move as usize] = index_to_replace;
      },
    );

    self.free_ids.extend(ids);

    BulkRemoveResp {
      items,
      indices: indices_to_replace,
    }
  }
}
