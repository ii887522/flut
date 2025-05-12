#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod app;
mod atlases;
mod audio;
mod batch;
mod batches;
mod buffers;
pub mod clock;
pub mod collections;
mod consts;
mod engine;
pub mod gfx;
mod graphics_pipeline;
mod images;
pub mod models;
mod shader;
pub mod transition;
pub mod widgets;

use ash::{
  Device,
  vk::{
    self, AccessFlags, Buffer, BufferImageCopy, CommandBuffer, DependencyFlags, Image,
    ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, PipelineStageFlags,
  },
};
use rayon::prelude::*;
use std::{
  mem,
  ops::{Bound, RangeBounds},
  ptr,
  sync::{Arc, atomic::AtomicU32},
};

pub use app::App;
pub use app::AppConfig;
pub use clock::Clock;
pub use engine::Engine;
pub use transition::Transition;

static APP_SIZE: (AtomicU32, AtomicU32) = (AtomicU32::new(0), AtomicU32::new(0));

const unsafe fn as_bytes<T>(from: &T) -> &[u8] {
  unsafe { &*ptr::slice_from_raw_parts(from as *const _ as *const _, mem::size_of::<T>()) }
}

pub fn par_swap_remove<T: Send>(vec: &mut Vec<T>, indices: impl RangeBounds<usize>) {
  let start_index = match indices.start_bound() {
    Bound::Included(&start_index) => start_index,
    Bound::Excluded(&start_index) => start_index + 1,
    Bound::Unbounded => 0,
  };

  let end_index = match indices.end_bound() {
    Bound::Included(&end_index) => end_index,
    Bound::Excluded(&end_index) => end_index - 1,
    Bound::Unbounded => vec.len() - 1,
  };

  let index_count = end_index - start_index + 1;
  let vec_start_index = start_index.max(vec.len() - index_count);

  if start_index == vec_start_index {
    // If remove last elements from vec, can simply remove
    vec.truncate(start_index);
  } else if end_index + 1 >= vec_start_index {
    // If remove elements from vec leaving few elements behind, Vec::drain() is sufficiently fast
    vec.par_drain(indices);
  } else {
    // Perform operation similar to Vec::swap_remove() but for a range of elements removal
    unsafe {
      ptr::copy_nonoverlapping(
        vec[vec_start_index..].as_ptr(),
        vec[start_index..=end_index].as_mut_ptr(),
        index_count,
      );
    }

    vec.truncate(vec_start_index);
  }
}

const fn pack_color(color: (u8, u8, u8, u8)) -> u32 {
  ((color.0 as u32) << 24) | ((color.1 as u32) << 16) | ((color.2 as u32) << 8) | (color.3 as u32)
}

const fn unpack_color(color: u32) -> (u8, u8, u8, u8) {
  (
    (color >> 24) as u8,
    (color >> 16) as u8,
    (color >> 8) as u8,
    color as u8,
  )
}

const fn map(from: f32, min_from: f32, max_from: f32, min_to: f32, max_to: f32) -> f32 {
  min_to + (from - min_from) * (max_to - min_to) / (max_from - min_from)
}

fn record_copy_buffer_to_image_commands(
  device: Arc<Device>,
  command_buffer: CommandBuffer,
  buffer: Buffer,
  image: Image,
  regions: &[BufferImageCopy],
  graphics_queue_family_index: u32,
  transfer_queue_family_index: u32,
  old_layout: ImageLayout,
) {
  let write_src_access_mask = match old_layout {
    ImageLayout::SHADER_READ_ONLY_OPTIMAL => AccessFlags::SHADER_READ,
    ImageLayout::UNDEFINED => AccessFlags::empty(),
    old_layout => unimplemented!("{old_layout:?} is not supported"),
  };

  let write_src_queue_family_index = match old_layout {
    ImageLayout::SHADER_READ_ONLY_OPTIMAL => graphics_queue_family_index,
    ImageLayout::UNDEFINED => vk::QUEUE_FAMILY_IGNORED,
    old_layout => unimplemented!("{old_layout:?} is not supported"),
  };

  let write_dst_queue_family_index = match old_layout {
    ImageLayout::UNDEFINED => vk::QUEUE_FAMILY_IGNORED,
    _ => transfer_queue_family_index,
  };

  let write_src_stage_mask = match old_layout {
    ImageLayout::SHADER_READ_ONLY_OPTIMAL => PipelineStageFlags::FRAGMENT_SHADER,
    ImageLayout::UNDEFINED => PipelineStageFlags::TOP_OF_PIPE,
    old_layout => unimplemented!("{old_layout:?} is not supported"),
  };

  let write_image_memory_barrier = ImageMemoryBarrier {
    src_access_mask: write_src_access_mask,
    dst_access_mask: AccessFlags::TRANSFER_WRITE,
    old_layout,
    new_layout: ImageLayout::TRANSFER_DST_OPTIMAL,
    src_queue_family_index: write_src_queue_family_index,
    dst_queue_family_index: write_dst_queue_family_index,
    image,
    subresource_range: ImageSubresourceRange {
      aspect_mask: ImageAspectFlags::COLOR,
      base_mip_level: 0,
      level_count: 1,
      base_array_layer: 0,
      layer_count: 1,
    },
    ..Default::default()
  };

  let read_image_memory_barrier = ImageMemoryBarrier {
    src_access_mask: AccessFlags::TRANSFER_WRITE,
    dst_access_mask: AccessFlags::SHADER_READ,
    old_layout: ImageLayout::TRANSFER_DST_OPTIMAL,
    new_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    src_queue_family_index: transfer_queue_family_index,
    dst_queue_family_index: graphics_queue_family_index,
    image,
    subresource_range: ImageSubresourceRange {
      aspect_mask: ImageAspectFlags::COLOR,
      base_mip_level: 0,
      level_count: 1,
      base_array_layer: 0,
      layer_count: 1,
    },
    ..Default::default()
  };

  unsafe {
    device.cmd_pipeline_barrier(
      command_buffer,
      write_src_stage_mask,
      PipelineStageFlags::TRANSFER,
      DependencyFlags::empty(),
      &[],
      &[],
      &[write_image_memory_barrier],
    );

    device.cmd_copy_buffer_to_image(
      command_buffer,
      buffer,
      image,
      ImageLayout::TRANSFER_DST_OPTIMAL,
      regions,
    );

    device.cmd_pipeline_barrier(
      command_buffer,
      PipelineStageFlags::TRANSFER,
      PipelineStageFlags::FRAGMENT_SHADER,
      DependencyFlags::empty(),
      &[],
      &[],
      &[read_image_memory_barrier],
    );
  }
}
