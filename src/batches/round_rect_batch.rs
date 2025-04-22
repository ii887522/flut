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
use std::{cell::RefCell, rc::Rc};

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct PushConstant {
  camera_size: (f32, f32),
  mesh_buffer_addr: DeviceAddress,
}

pub(crate) struct RoundRectBatch<'a> {
  batch: Batch<'a, RoundRectPart>,
  round_rect_ids: SparseVec<Vec<u16>>,
}

impl RoundRectBatch<'_> {
  pub(crate) fn new(
    device: Rc<Device>,
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
  ) {
    let push_const = PushConstant {
      camera_size: (surface_extent.width as _, surface_extent.height as _),
      mesh_buffer_addr: self.batch.mesh_buffer_addr,
    };

    self
      .batch
      .record_draw_commands(command_buffer, &[], push_const);
  }

  fn batch_add_part(&mut self, round_rect_parts: Vec<RoundRectPart>) -> Vec<u16> {
    self.batch.batch_add(round_rect_parts)
  }

  fn batch_remove_part(&mut self, ids: &[u16]) {
    self.batch.batch_remove(ids);
  }

  pub(crate) fn add(&mut self, round_rect: RoundRect) -> u16 {
    let ids = self.batch_add_part(Vec::from(round_rect));
    self.round_rect_ids.push(ids)
  }

  pub(crate) fn remove(&mut self, id: u16) {
    let ids = self.round_rect_ids.remove(id);
    self.batch_remove_part(&ids);
  }

  pub(crate) fn clear(&mut self) {
    self.batch.clear();
  }
}
