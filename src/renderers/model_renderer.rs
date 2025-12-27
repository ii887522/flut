use crate::{
  collections::{SparseSet, sparse_set::Id},
  models::Write,
  pipelines::{self, CreatedPipeline, CreatingPipeline, Model},
  renderers::MAX_IN_FLIGHT_FRAME_COUNT,
  utils,
};
use ash::vk::{self, Handle};
use rayon::prelude::*;
use std::{collections::VecDeque, mem, ptr, slice};

const MIN_SEQ_LEN: usize = 1024;

pub(super) trait State {
  type Model: pipelines::Model;
}

pub(super) struct Creating<Model: pipelines::Model> {
  pipeline: Model::CreatingPipeline,
}

impl<Model: pipelines::Model> State for Creating<Model> {
  type Model = Model;
}

pub(super) struct Created<Model: pipelines::Model> {
  pipeline: Model::CreatedPipeline,
}

impl<Model: pipelines::Model> State for Created<Model> {
  type Model = Model;
}

pub(super) struct ModelRenderer<S: State> {
  models: SparseSet<S::Model>,
  writes_queue: VecDeque<Vec<Write>>,
  model_capacity: usize,
  state: S,
}

impl<Model: pipelines::Model> ModelRenderer<Creating<Model>> {
  pub(super) fn new(device: &ash::Device, model_capacity: usize) -> Self {
    Self {
      models: SparseSet::with_capacity(model_capacity),
      writes_queue: VecDeque::from_iter([vec![]]),
      model_capacity,
      state: Creating {
        pipeline: Model::CreatingPipeline::new(device),
      },
    }
  }

  pub(super) fn finish(
    self,
    device: &ash::Device,
    render_pass: vk::RenderPass,
    pipeline_cache: vk::PipelineCache,
    swapchain_image_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
  ) -> ModelRenderer<Created<Model>> {
    ModelRenderer {
      models: self.models,
      writes_queue: self.writes_queue,
      model_capacity: self.model_capacity,
      state: Created {
        pipeline: self.state.pipeline.finish(
          device,
          render_pass,
          pipeline_cache,
          swapchain_image_extent,
          msaa_samples,
        ),
      },
    }
  }

  #[inline]
  pub(super) fn get_descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
    self.state.pipeline.get_descriptor_set_layout()
  }

  pub(super) fn drop(self, device: &ash::Device) {
    self.state.pipeline.drop(device);
  }
}

impl<Model: pipelines::Model> ModelRenderer<Created<Model>> {
  #[inline]
  pub(super) const fn get_model_capacity(&self) -> usize {
    self.model_capacity
  }

  pub(super) fn flush_writes(&mut self, model_buffer_data: *mut Model) {
    let writes = self.writes_queue.back_mut().unwrap();
    utils::coalesce_writes(writes);

    let mut queued_writes = self
      .writes_queue
      .iter()
      .flat_map(|writes| writes.clone())
      .collect();

    utils::coalesce_writes(&mut queued_writes);
    let models = self.models.get_items();

    unsafe {
      for write in queued_writes {
        ptr::copy_nonoverlapping(
          models.as_ptr().add(write.index as _),
          model_buffer_data.add(write.index as _),
          write.len as _,
        );
      }
    }

    self.writes_queue.push_back(vec![]);

    if self.writes_queue.len() > MAX_IN_FLIGHT_FRAME_COUNT {
      self.writes_queue.pop_front();
    }
  }

  pub(super) fn record_draw_commands(
    &self,
    device: &ash::Device,
    graphics_command_buffer: vk::CommandBuffer,
    descriptor_set: vk::DescriptorSet,
    push_consts: &Model::PushConsts,
  ) {
    if self.models.is_empty() {
      return;
    }

    unsafe {
      device.cmd_bind_pipeline(
        graphics_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.state.pipeline.get_pipeline(),
      );

      if !descriptor_set.is_null() {
        device.cmd_bind_descriptor_sets(
          graphics_command_buffer,
          vk::PipelineBindPoint::GRAPHICS,
          self.state.pipeline.get_pipeline_layout(),
          0,
          &[descriptor_set],
          &[],
        );
      }

      device.cmd_push_constants(
        graphics_command_buffer,
        self.state.pipeline.get_pipeline_layout(),
        vk::ShaderStageFlags::VERTEX,
        0,
        slice::from_raw_parts(
          push_consts as *const _ as *const _,
          mem::size_of::<Model::PushConsts>(),
        ),
      );

      device.cmd_draw(
        graphics_command_buffer,
        (self.models.len() * Model::get_vertex_count()) as _,
        1,
        0,
        0,
      );
    }
  }

  pub(super) fn on_swapchain_suboptimal(self) -> ModelRenderer<Creating<Model>> {
    ModelRenderer {
      models: self.models,
      writes_queue: self.writes_queue,
      model_capacity: self.model_capacity,
      state: Creating {
        pipeline: self.state.pipeline.on_swapchain_suboptimal(),
      },
    }
  }

  pub(super) fn drop(self, device: &ash::Device) {
    self.state.pipeline.drop(device);
  }
}

impl<S: State> ModelRenderer<S> {
  pub(super) fn add_model(&mut self, model: S::Model) -> Id {
    debug_assert!(
      self.models.len() < self.model_capacity,
      "{model_name} capacity exceeded: {} < {}",
      self.models.len(),
      self.model_capacity,
      model_name = S::Model::get_name(),
    );

    let add_resp = self.models.add(model);
    let writes = self.writes_queue.back_mut().unwrap();

    writes.push(Write {
      index: add_resp.index,
      len: 1,
    });

    add_resp.id
  }

  pub(super) fn update_model(&mut self, id: Id, model: S::Model) {
    let update_resp = self.models.update(id, model);
    let writes = self.writes_queue.back_mut().unwrap();

    writes.push(Write {
      index: update_resp.index,
      len: 1,
    });
  }

  pub(super) fn remove_model(&mut self, id: Id) {
    let remove_resp = self.models.remove(id);

    let Some(index) = remove_resp.index else {
      return;
    };

    let writes = self.writes_queue.back_mut().unwrap();
    writes.push(Write { index, len: 1 });
  }
}

impl<S: State> ModelRenderer<S>
where
  S::Model: Send,
{
  pub(super) fn bulk_add_models(&mut self, models: Box<[S::Model]>) -> Box<[Id]> {
    debug_assert!(
      self.models.len() + models.len() <= self.model_capacity,
      "{model_name} capacity exceeded: {} + {} <= {}",
      self.models.len(),
      models.len(),
      self.model_capacity,
      model_name = S::Model::get_name(),
    );

    let bulk_add_resp = self.models.bulk_add(models);
    let writes = self.writes_queue.back_mut().unwrap();

    writes.push(Write {
      index: bulk_add_resp.index,
      len: bulk_add_resp.len,
    });

    bulk_add_resp.ids
  }

  pub(super) fn bulk_update_models(&mut self, updates: Box<[(Id, S::Model)]>) {
    let bulk_update_resp = self.models.bulk_update(updates);
    let writes = self.writes_queue.back_mut().unwrap();

    writes.par_extend(
      bulk_update_resp
        .indices
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(|index| Write { index, len: 1 }),
    );
  }
}

impl<S: State> ModelRenderer<S>
where
  S::Model: Clone + Send,
{
  pub(super) fn bulk_remove_models(&mut self, ids: &[Id]) {
    let bulk_remove_resp = self.models.bulk_remove(ids);
    let writes = self.writes_queue.back_mut().unwrap();

    writes.par_extend(
      bulk_remove_resp
        .indices
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(|index| Write { index, len: 1 }),
    );
  }
}
