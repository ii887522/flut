use crate::{
  batch::Batch,
  collections::SparseVec,
  models::{RoundRect, RoundRectPart},
};
use ash::{
  Device,
  vk::{CommandBuffer, DeviceAddress, Extent2D, RenderPass},
};
use gpu_allocator::vulkan::Allocator;
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct PushConstant {
  camera_size: (f32, f32),
  pixel_size: f32,
  pad: f32,
  mesh_buffer_addr: DeviceAddress,
}

pub(crate) struct RoundRectBatch {
  batch: Batch<RoundRectPart>,
  round_rect_ids: SparseVec<Vec<u16>>,
}

impl RoundRectBatch {
  pub(crate) fn new(
    device: Arc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    cap: usize,
  ) -> Self {
    let batch = Batch::new::<PushConstant>(
      device.clone(),
      memory_allocator.clone(),
      cap,
      include_bytes!("../../target/shaders/round_rect.vert.spv"),
      include_bytes!("../../target/shaders/round_rect.frag.spv"),
      &[],
      "round_rect_buffer",
    );

    Self {
      batch,
      round_rect_ids: SparseVec::new(),
    }
  }

  pub(crate) fn on_swapchain_suboptimal(
    &mut self,
    surface_extent: Extent2D,
    render_pass: RenderPass,
  ) {
    self
      .batch
      .on_swapchain_suboptimal(surface_extent, render_pass);
  }

  pub(crate) fn record_draw_commands(
    &self,
    command_buffer: CommandBuffer,
    surface_extent: Extent2D,
    pixel_size: f32,
  ) {
    let push_const = PushConstant {
      camera_size: (surface_extent.width as _, surface_extent.height as _),
      pixel_size,
      pad: 0.0,
      mesh_buffer_addr: self.batch.mesh_buffer_addr,
    };

    self
      .batch
      .record_draw_commands(command_buffer, &[], push_const);
  }

  fn batch_add_part(&mut self, round_rect_parts: Vec<RoundRectPart>) -> Vec<u16> {
    self.batch.batch_add(round_rect_parts)
  }

  fn batch_update_part(&self, ids: &[u16], round_rect_parts: Vec<RoundRectPart>) {
    self.batch.batch_update(ids, round_rect_parts);
  }

  fn batch_remove_part(&mut self, ids: &[u16]) {
    self.batch.batch_remove(ids);
  }

  pub(crate) fn add(&mut self, round_rect: RoundRect) -> u16 {
    let ids = self.batch_add_part(Vec::from(round_rect));
    self.round_rect_ids.push(ids)
  }

  pub(crate) fn update(&self, id: u16, round_rect: RoundRect) {
    let ids = self.round_rect_ids[id].borrow();
    self.batch_update_part(&ids, Vec::from(round_rect));
  }

  pub(crate) fn remove(&mut self, id: u16) {
    let ids = self.round_rect_ids.remove(id);
    self.batch_remove_part(&ids);
  }

  pub(crate) fn clear(&mut self) {
    self.batch.clear();
  }
}
