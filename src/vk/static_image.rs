use crate::vk::Device;
use ash::vk;
use std::{ptr, rc::Rc};
use vk_mem::Alloc;

pub(super) struct Creating {
  staging_buffer: vk::Buffer,
  staging_alloc: vk_mem::Allocation,
}

pub(super) struct Created;

pub(super) struct StaticImage<State> {
  device: Rc<Device>,
  vk_allocator: Rc<vk_mem::Allocator>,
  image: vk::Image,
  view: vk::ImageView,
  vk_alloc: vk_mem::Allocation,
  state: State,
}

impl StaticImage<Creating> {
  pub(super) fn new(
    vk_device: Rc<Device>,
    vk_allocator: Rc<vk_mem::Allocator>,
    pixels: &[u8],
    regions: &[vk::BufferImageCopy2<'_>],
    format: vk::Format,
    size: (u32, u32),
    transfer_command_buffer: vk::CommandBuffer,
    transfer_queue_family_index: u32,
    graphics_queue_family_index: u32,
  ) -> Self {
    let device = vk_device.get();

    let staging_buffer_create_info = vk::BufferCreateInfo {
      size: pixels.len() as _,
      usage: vk::BufferUsageFlags::TRANSFER_SRC,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let staging_alloc_create_info = vk_mem::AllocationCreateInfo {
      flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
        | vk_mem::AllocationCreateFlags::MAPPED,
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      priority: 1.0,
      ..Default::default()
    };

    let (staging_buffer, staging_alloc) = unsafe {
      vk_allocator
        .create_buffer(&staging_buffer_create_info, &staging_alloc_create_info)
        .unwrap()
    };

    let staging_alloc_info = vk_allocator.get_allocation_info(&staging_alloc);
    let staging_mapped_data = staging_alloc_info.mapped_data;

    unsafe {
      ptr::copy_nonoverlapping(pixels.as_ptr(), staging_mapped_data as *mut _, pixels.len());
    }

    let image_create_info = vk::ImageCreateInfo {
      image_type: vk::ImageType::TYPE_2D,
      format,
      extent: vk::Extent3D {
        width: size.0,
        height: size.1,
        depth: 1,
      },
      mip_levels: 1,
      array_layers: 1,
      samples: vk::SampleCountFlags::TYPE_1,
      tiling: vk::ImageTiling::OPTIMAL,
      usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      initial_layout: vk::ImageLayout::UNDEFINED,
      ..Default::default()
    };

    let alloc_create_info = vk_mem::AllocationCreateInfo {
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      priority: 1.0,
      ..Default::default()
    };

    let (image, vk_alloc) = unsafe {
      vk_allocator
        .create_image(&image_create_info, &alloc_create_info)
        .unwrap()
    };

    let image_view_create_info = vk::ImageViewCreateInfo {
      image,
      view_type: vk::ImageViewType::TYPE_2D,
      format,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    let image_view = unsafe {
      device
        .create_image_view(&image_view_create_info, None)
        .unwrap()
    };

    let before_copy_image_memory_barrier = vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::empty(),
      dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
      old_layout: vk::ImageLayout::UNDEFINED,
      new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    let copy_buffer_to_image_info = vk::CopyBufferToImageInfo2 {
      src_buffer: staging_buffer,
      dst_image: image,
      dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      region_count: regions.len() as _,
      p_regions: regions.as_ptr(),
      ..Default::default()
    };

    let after_copy_image_memory_barrier = vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
      dst_access_mask: vk::AccessFlags::SHADER_READ,
      old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      src_queue_family_index: transfer_queue_family_index,
      dst_queue_family_index: graphics_queue_family_index,
      image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    unsafe {
      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &[before_copy_image_memory_barrier],
      );

      device.cmd_copy_buffer_to_image2(transfer_command_buffer, &copy_buffer_to_image_info);

      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &[after_copy_image_memory_barrier],
      );
    }

    Self {
      device: vk_device,
      vk_allocator,
      image,
      view: image_view,
      vk_alloc,
      state: Creating {
        staging_buffer,
        staging_alloc,
      },
    }
  }

  pub(super) fn finish(mut self) -> StaticImage<Created> {
    unsafe {
      self
        .vk_allocator
        .destroy_buffer(self.state.staging_buffer, &mut self.state.staging_alloc);
    }

    StaticImage {
      device: self.device,
      vk_allocator: self.vk_allocator,
      image: self.image,
      view: self.view,
      vk_alloc: self.vk_alloc,
      state: Created,
    }
  }
}

impl StaticImage<Created> {
  pub(super) fn drop(mut self) {
    let device = self.device.get();

    unsafe {
      device.destroy_image_view(self.view, None);

      self
        .vk_allocator
        .destroy_image(self.image, &mut self.vk_alloc);
    }
  }
}

impl<State> StaticImage<State> {
  pub(super) const fn get_view(&self) -> vk::ImageView {
    self.view
  }
}
