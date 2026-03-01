use crate::consts;
use ash::vk;
use std::{ffi::c_void, iter, ptr};
use vk_mem::Alloc as _;

pub struct SampledImage {
  staging_buffer: vk::Buffer,
  staging_buffer_alloc: vk_mem::Allocation,
  staging_buffer_data: *mut c_void,
  image: vk::Image,
  alloc: vk_mem::Allocation,
  image_views: Box<[vk::ImageView]>,
  transfer_command_pools: Box<[vk::CommandPool]>,
  transfer_command_buffers: Box<[vk::CommandBuffer]>,
  size: usize,
  read_index: usize,
}

impl SampledImage {
  pub(super) fn new(
    vk_device: &ash::Device,
    vk_allocator: &vk_mem::Allocator,
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
    size: usize,
    extent: vk::Extent2D,
  ) -> (Self, vk::CommandBuffer) {
    let vk::Extent2D { width, height } = extent;

    let staging_buffer_create_info = vk::BufferCreateInfo {
      size: (consts::MAX_IN_FLIGHT_FRAME_COUNT * size) as u64,
      usage: vk::BufferUsageFlags::TRANSFER_SRC,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let staging_buffer_alloc_create_info = vk_mem::AllocationCreateInfo {
      flags: vk_mem::AllocationCreateFlags::MAPPED
        | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
      usage: vk_mem::MemoryUsage::AutoPreferHost,
      priority: 1.0,
      ..Default::default()
    };

    let (staging_buffer, staging_buffer_alloc) = unsafe {
      vk_allocator
        .create_buffer(
          &staging_buffer_create_info,
          &staging_buffer_alloc_create_info,
        )
        .unwrap()
    };

    let staging_buffer_alloc_info = vk_allocator.get_allocation_info2(&staging_buffer_alloc);
    let staging_buffer_data = staging_buffer_alloc_info.allocation_info.mapped_data;

    let image_create_info = vk::ImageCreateInfo {
      image_type: vk::ImageType::TYPE_2D,
      format: vk::Format::R8_UNORM,
      extent: vk::Extent3D {
        width,
        height,
        depth: 1,
      },
      mip_levels: 1,
      array_layers: consts::MAX_IN_FLIGHT_FRAME_COUNT as u32,
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

    let (image, alloc) = unsafe {
      vk_allocator
        .create_image(&image_create_info, &alloc_create_info)
        .unwrap()
    };

    let image_views = (0..consts::MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|index| {
        let image_view_create_info = vk::ImageViewCreateInfo {
          image,
          view_type: vk::ImageViewType::TYPE_2D,
          format: vk::Format::R8_UNORM,
          subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: index as u32,
            layer_count: 1,
          },
          ..Default::default()
        };

        unsafe {
          vk_device
            .create_image_view(&image_view_create_info, None)
            .unwrap()
        }
      })
      .collect::<Box<_>>();

    let transfer_command_pools = iter::repeat_with(|| {
      let command_pool_create_info = vk::CommandPoolCreateInfo {
        flags: vk::CommandPoolCreateFlags::TRANSIENT,
        queue_family_index: transfer_queue_family_index,
        ..Default::default()
      };

      unsafe {
        vk_device
          .create_command_pool(&command_pool_create_info, None)
          .unwrap()
      }
    })
    .take(consts::MAX_IN_FLIGHT_FRAME_COUNT + 1)
    .collect::<Box<_>>();

    let transfer_command_buffers = transfer_command_pools
      .iter()
      .map(|&command_pool| {
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
          command_pool,
          level: vk::CommandBufferLevel::PRIMARY,
          command_buffer_count: 1,
          ..Default::default()
        };

        unsafe {
          vk_device
            .allocate_command_buffers(&command_buffer_alloc_info)
            .unwrap()[0]
        }
      })
      .collect::<Box<_>>();

    let transfer_command_buffer = transfer_command_buffers[consts::MAX_IN_FLIGHT_FRAME_COUNT];

    let transfer_command_buffer_begin_info = vk::CommandBufferBeginInfo {
      flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    unsafe {
      vk_device
        .begin_command_buffer(transfer_command_buffer, &transfer_command_buffer_begin_info)
        .unwrap();
    }

    let image_memory_barrier = vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::empty(),
      dst_access_mask: vk::AccessFlags::SHADER_READ,
      old_layout: vk::ImageLayout::UNDEFINED,
      new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      src_queue_family_index: transfer_queue_family_index,
      dst_queue_family_index: graphics_queue_family_index,
      image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: consts::MAX_IN_FLIGHT_FRAME_COUNT as u32,
      },
      ..Default::default()
    };

    unsafe {
      vk_device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &[image_memory_barrier],
      );
    }

    unsafe {
      vk_device
        .end_command_buffer(transfer_command_buffer)
        .unwrap();
    }

    (
      Self {
        staging_buffer,
        staging_buffer_alloc,
        staging_buffer_data,
        image,
        alloc,
        image_views,
        transfer_command_pools,
        transfer_command_buffers,
        size,
        read_index: 0,
      },
      transfer_command_buffer,
    )
  }

  #[inline]
  pub(super) const fn get_image_views(&self) -> &[vk::ImageView] {
    &self.image_views
  }

  #[inline]
  pub(super) const fn get_read_index(&self) -> usize {
    self.read_index
  }

  pub(super) fn write(
    &mut self,
    vk_device: &ash::Device,
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
    pixels: &[u8],
    regions: &[vk::BufferImageCopy2],
  ) -> vk::CommandBuffer {
    let write_index = (self.read_index + 1) % consts::MAX_IN_FLIGHT_FRAME_COUNT;
    let write_offset = write_index * self.size;
    let dst = unsafe { self.staging_buffer_data.byte_add(write_offset).cast() };

    unsafe {
      ptr::copy_nonoverlapping(pixels.as_ptr(), dst, pixels.len());
    }

    let transfer_command_pool = self.transfer_command_pools[write_index];
    let transfer_command_buffer = self.transfer_command_buffers[write_index];

    unsafe {
      vk_device
        .reset_command_pool(transfer_command_pool, vk::CommandPoolResetFlags::empty())
        .unwrap();
    }

    let transfer_command_buffer_begin_info = vk::CommandBufferBeginInfo {
      flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    unsafe {
      vk_device
        .begin_command_buffer(transfer_command_buffer, &transfer_command_buffer_begin_info)
        .unwrap();
    }

    let image_memory_barrier = vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::SHADER_READ,
      dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
      old_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      src_queue_family_index: graphics_queue_family_index,
      dst_queue_family_index: transfer_queue_family_index,
      image: self.image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: write_index as u32,
        layer_count: 1,
      },
      ..Default::default()
    };

    unsafe {
      vk_device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &[image_memory_barrier],
      );
    }

    let regions = regions
      .iter()
      .map(|&region| vk::BufferImageCopy2 {
        buffer_offset: region.buffer_offset + write_offset as u64,
        image_subresource: vk::ImageSubresourceLayers {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          mip_level: 0,
          base_array_layer: write_index as u32,
          layer_count: 1,
        },
        ..region
      })
      .collect::<Box<_>>();

    let copy_buffer_to_image_info = vk::CopyBufferToImageInfo2 {
      src_buffer: self.staging_buffer,
      dst_image: self.image,
      dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      region_count: regions.len().try_into().unwrap(),
      p_regions: regions.as_ptr(),
      ..Default::default()
    };

    unsafe {
      vk_device.cmd_copy_buffer_to_image2(transfer_command_buffer, &copy_buffer_to_image_info);
    }

    let image_memory_barrier = vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
      dst_access_mask: vk::AccessFlags::SHADER_READ,
      old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      src_queue_family_index: transfer_queue_family_index,
      dst_queue_family_index: graphics_queue_family_index,
      image: self.image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: write_index as u32,
        layer_count: 1,
      },
      ..Default::default()
    };

    unsafe {
      vk_device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &[image_memory_barrier],
      );
    }

    unsafe {
      vk_device
        .end_command_buffer(transfer_command_buffer)
        .unwrap();
    }

    transfer_command_buffer
  }

  #[inline]
  pub(super) const fn done_write(&mut self) {
    self.read_index = (self.read_index + 1) % consts::MAX_IN_FLIGHT_FRAME_COUNT;
  }

  pub(super) fn drop(mut self, vk_device: &ash::Device, vk_allocator: &vk_mem::Allocator) {
    unsafe {
      self
        .transfer_command_pools
        .iter()
        .for_each(|&command_pool| vk_device.destroy_command_pool(command_pool, None));
    }
    unsafe {
      self
        .image_views
        .iter()
        .for_each(|&image_view| vk_device.destroy_image_view(image_view, None));
    }
    unsafe {
      vk_allocator.destroy_image(self.image, &mut self.alloc);
    }
    unsafe {
      vk_allocator.destroy_buffer(self.staging_buffer, &mut self.staging_buffer_alloc);
    }
  }
}
