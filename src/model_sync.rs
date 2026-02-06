use crate::{
  collections::sparse_set::{
    AddResp, BulkAddResp, BulkRemoveResp, BulkUpdateResp, RemoveResp, SparseSet, UpdateResp,
  },
  consts,
  models::range::Range,
  storage_buffer::StorageBuffer,
  utils,
};
use ash::vk;
use std::collections::VecDeque;

pub enum SyncStatus {
  Unchanged,
  Changed(Option<vk::CommandBuffer>),
}

pub struct ModelSync<Model> {
  models: SparseSet<Model>,
  changeset_queue: VecDeque<Vec<Range>>,
}

impl<Model> ModelSync<Model> {
  pub(super) fn new(model_capacity: usize) -> Self {
    Self {
      models: SparseSet::with_capacity(model_capacity),
      changeset_queue: VecDeque::from_iter([vec![]]),
    }
  }

  #[inline]
  pub(super) const fn get_model_count(&self) -> usize {
    self.models.len()
  }

  pub(super) fn sync_to(
    &mut self,
    model_buffer: &mut StorageBuffer,
    vk_device: &ash::Device,
  ) -> SyncStatus {
    let changeset = self.changeset_queue.back_mut().unwrap();
    utils::coalesce_ranges(changeset);

    let mut all_changeset = self
      .changeset_queue
      .iter()
      .flatten()
      .copied()
      .collect::<Vec<_>>();

    utils::coalesce_ranges(&mut all_changeset);

    if all_changeset.is_empty() {
      return SyncStatus::Unchanged;
    }

    let transfer_command_buffer =
      model_buffer.write(vk_device, self.models.get_items(), &all_changeset);

    if self.changeset_queue.len() >= consts::MAX_IN_FLIGHT_FRAME_COUNT {
      self.changeset_queue.pop_front();
    }

    self.changeset_queue.push_back(vec![]);
    SyncStatus::Changed(transfer_command_buffer)
  }

  pub(super) fn add_model(&mut self, model: Model) -> u32 {
    let model_count = self.models.len().try_into().unwrap();
    let AddResp { id } = self.models.add(model);
    let changeset = self.changeset_queue.back_mut().unwrap();

    changeset.push(Range {
      start: model_count,
      end: model_count + 1,
    });

    id
  }

  pub(super) fn update_model(&mut self, id: u32, model: Model) {
    let UpdateResp { index } = self.models.update(id, model);
    let changeset = self.changeset_queue.back_mut().unwrap();

    changeset.push(Range {
      start: index,
      end: index + 1,
    });
  }

  pub(super) fn remove_model(&mut self, id: u32) -> Model {
    let RemoveResp { item, index } = self.models.remove(id);

    if let Some(index) = index {
      let changeset = self.changeset_queue.back_mut().unwrap();

      changeset.push(Range {
        start: index,
        end: index + 1,
      });
    }

    item
  }

  pub(super) fn bulk_add_models(&mut self, models: Box<[Model]>) -> Box<[u32]> {
    let model_count = self.models.len();
    let BulkAddResp { ids } = self.models.bulk_add(models);
    let changeset = self.changeset_queue.back_mut().unwrap();

    changeset.push(Range {
      start: model_count.try_into().unwrap(),
      end: (model_count + ids.len()).try_into().unwrap(),
    });

    ids
  }

  pub(super) fn bulk_update_models(&mut self, ids: &[u32], models: Box<[Model]>) {
    let BulkUpdateResp { indices } = self.models.bulk_update(ids, models);
    let changeset = self.changeset_queue.back_mut().unwrap();

    changeset.extend(indices.into_iter().map(|index| Range {
      start: index,
      end: index + 1,
    }));
  }
}

impl<Model: Clone> ModelSync<Model> {
  pub(super) fn bulk_remove_models(&mut self, ids: &[u32]) -> Box<[Model]> {
    let BulkRemoveResp { items, indices } = self.models.bulk_remove(ids);
    let changeset = self.changeset_queue.back_mut().unwrap();

    changeset.extend(indices.into_iter().map(|index| Range {
      start: index,
      end: index + 1,
    }));

    items
  }
}
