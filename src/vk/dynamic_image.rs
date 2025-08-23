use crate::vk::Device;
use ash::vk;
use rayon::prelude::*;
use std::{collections::VecDeque, ffi::c_void, rc::Rc};
use vk_mem::Alloc;

pub(super) struct DynamicImage {
  device: Rc<Device>,
  vk_allocator: Rc<vk_mem::Allocator>,
  staging_buffer: vk::Buffer,
  staging_vk_alloc: vk_mem::Allocation,
  staging_mapped_data: *mut c_void,
  image: vk::Image,
  vk_alloc: vk_mem::Allocation,
  views: Box<[vk::ImageView]>,
  max_bytes: usize,
  read_view_index: usize,
}

impl DynamicImage {
  pub(super) fn new(
    vk_device: Rc<Device>,
    vk_allocator: Rc<vk_mem::Allocator>,
    format: vk::Format,
    size: (u32, u32),
    max_bytes: usize,
    transfer_command_buffer: vk::CommandBuffer,
  ) -> Self {
    let device = vk_device.get();

    let staging_buffer_create_info = vk::BufferCreateInfo {
      size: (max_bytes * crate::consts::SUB_DYNAMIC_BUFFER_COUNT) as _,
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

    let (staging_buffer, staging_vk_alloc) = unsafe {
      vk_allocator
        .create_buffer(&staging_buffer_create_info, &staging_alloc_create_info)
        .unwrap()
    };

    let staging_alloc_info = vk_allocator.get_allocation_info(&staging_vk_alloc);
    let staging_mapped_data = staging_alloc_info.mapped_data;

    let image_create_info = vk::ImageCreateInfo {
      image_type: vk::ImageType::TYPE_2D,
      format,
      extent: vk::Extent3D {
        width: size.0,
        height: size.1,
        depth: 1,
      },
      mip_levels: 1,
      array_layers: ((crate::consts::SUB_DYNAMIC_BUFFER_COUNT << 1) - 1) as _,
      samples: vk::SampleCountFlags::TYPE_1,
      tiling: vk::ImageTiling::OPTIMAL,
      usage: vk::ImageUsageFlags::TRANSFER_SRC
        | vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::SAMPLED,
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

    let image_view_create_infos = [
      vk::ImageViewCreateInfo {
        image,
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        subresource_range: vk::ImageSubresourceRange {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          base_mip_level: 0,
          level_count: 1,
          base_array_layer: 1,
          layer_count: 1,
        },
        ..Default::default()
      },
      vk::ImageViewCreateInfo {
        image,
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        subresource_range: vk::ImageSubresourceRange {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          base_mip_level: 0,
          level_count: 1,
          base_array_layer: 2,
          layer_count: 1,
        },
        ..Default::default()
      },
    ];

    let image_views = image_view_create_infos
      .iter()
      .map(|image_view_create_info| unsafe {
        device
          .create_image_view(image_view_create_info, None)
          .unwrap()
      })
      .collect::<Box<_>>();

    let image_memory_barriers = [
      vk::ImageMemoryBarrier {
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
      },
      vk::ImageMemoryBarrier {
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::SHADER_READ,
        old_layout: vk::ImageLayout::UNDEFINED,
        new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image,
        subresource_range: vk::ImageSubresourceRange {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          base_mip_level: 0,
          level_count: 1,
          base_array_layer: 1,
          layer_count: crate::consts::SUB_DYNAMIC_BUFFER_COUNT as _,
        },
        ..Default::default()
      },
    ];

    unsafe {
      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &image_memory_barriers,
      );
    }

    Self {
      device: vk_device,
      vk_allocator,
      staging_buffer,
      staging_vk_alloc,
      staging_mapped_data,
      image,
      vk_alloc,
      views: image_views,
      max_bytes,
      read_view_index: 0,
    }
  }

  pub(super) const fn get_staging_mapped_data(&self) -> *mut c_void {
    unsafe {
      self
        .staging_mapped_data
        .add(self.get_staging_buffer_offset())
    }
  }

  pub(super) const fn get_staging_buffer_offset(&self) -> usize {
    self.get_write_view_index() * self.max_bytes
  }

  pub(super) const fn get_views(&self) -> &[vk::ImageView] {
    &self.views
  }

  pub(super) const fn get_read_view_index(&self) -> usize {
    self.read_view_index
  }

  pub(super) const fn get_write_view_index(&self) -> usize {
    (self.read_view_index + 1) % crate::consts::SUB_DYNAMIC_BUFFER_COUNT
  }

  pub(super) const fn get_write_layer_index(&self) -> usize {
    crate::consts::SUB_DYNAMIC_BUFFER_COUNT - 1 + self.get_write_view_index()
  }

  pub(super) fn record_flush_commands(
    &mut self,
    transfer_command_buffer: vk::CommandBuffer,
    regions_queues: &mut VecDeque<Vec<vk::BufferImageCopy2<'_>>>,
    transfer_queue_family_index: u32,
    graphics_queue_family_index: u32,
  ) {
    let device = self.device.get();
    let base_array_layer = self.get_write_layer_index() as _;
    let mut before_flush_image_memory_barriers = Vec::with_capacity(2);

    let flush_aux_regions = if regions_queues.len() > 1 {
      before_flush_image_memory_barriers.push(vk::ImageMemoryBarrier {
        src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
        dst_access_mask: vk::AccessFlags::TRANSFER_READ,
        old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: self.image,
        subresource_range: vk::ImageSubresourceRange {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          base_mip_level: 0,
          level_count: 1,
          base_array_layer: 0,
          layer_count: 1,
        },
        ..Default::default()
      });

      let regions = regions_queues.front().unwrap();

      let regions = regions
        .par_iter()
        .map(|region| vk::ImageCopy2 {
          src_subresource: vk::ImageSubresourceLayers {
            base_array_layer: 0,
            ..region.image_subresource
          },
          src_offset: region.image_offset,
          dst_subresource: vk::ImageSubresourceLayers {
            base_array_layer,
            ..region.image_subresource
          },
          dst_offset: region.image_offset,
          extent: region.image_extent,
          ..Default::default()
        })
        .collect::<Box<_>>();

      Some(regions)
    } else {
      None
    };

    let copy_aux_to_image_info = flush_aux_regions
      .as_ref()
      .map(|regions| vk::CopyImageInfo2 {
        src_image: self.image,
        src_image_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        dst_image: self.image,
        dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        region_count: regions.len() as _,
        p_regions: regions.as_ptr(),
        ..Default::default()
      });

    before_flush_image_memory_barriers.push(vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::SHADER_READ,
      dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
      old_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      image: self.image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer,
        layer_count: 1,
      },
      ..Default::default()
    });

    let regions = regions_queues.back().unwrap();

    let copy_buffer_to_image_info = vk::CopyBufferToImageInfo2 {
      src_buffer: self.staging_buffer,
      dst_image: self.image,
      dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      region_count: regions.len() as _,
      p_regions: regions.as_ptr(),
      ..Default::default()
    };

    let mut aux_image_memory_barriers = Vec::with_capacity(2);

    if regions_queues.len() > 1 {
      aux_image_memory_barriers.push(vk::ImageMemoryBarrier {
        src_access_mask: vk::AccessFlags::TRANSFER_READ,
        dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
        old_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: self.image,
        subresource_range: vk::ImageSubresourceRange {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          base_mip_level: 0,
          level_count: 1,
          base_array_layer: 0,
          layer_count: 1,
        },
        ..Default::default()
      });
    }

    aux_image_memory_barriers.push(vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
      dst_access_mask: vk::AccessFlags::TRANSFER_READ,
      old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
      src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      image: self.image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer,
        layer_count: 1,
      },
      ..Default::default()
    });

    let regions = regions
      .par_iter()
      .map(|region| vk::ImageCopy2 {
        src_subresource: region.image_subresource,
        src_offset: region.image_offset,
        dst_subresource: vk::ImageSubresourceLayers {
          base_array_layer: 0,
          ..region.image_subresource
        },
        dst_offset: region.image_offset,
        extent: region.image_extent,
        ..Default::default()
      })
      .collect::<Box<_>>();

    let copy_image_to_aux_info = vk::CopyImageInfo2 {
      src_image: self.image,
      src_image_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
      dst_image: self.image,
      dst_image_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      region_count: regions.len() as _,
      p_regions: regions.as_ptr(),
      ..Default::default()
    };

    let after_flush_image_memory_barriers = [vk::ImageMemoryBarrier {
      src_access_mask: vk::AccessFlags::TRANSFER_READ,
      dst_access_mask: vk::AccessFlags::SHADER_READ,
      old_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
      new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      src_queue_family_index: transfer_queue_family_index,
      dst_queue_family_index: graphics_queue_family_index,
      image: self.image,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer,
        layer_count: 1,
      },
      ..Default::default()
    }];

    unsafe {
      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &before_flush_image_memory_barriers,
      );

      if let Some(copy_image_info) = copy_aux_to_image_info {
        device.cmd_copy_image2(transfer_command_buffer, &copy_image_info);
      }

      device.cmd_copy_buffer_to_image2(transfer_command_buffer, &copy_buffer_to_image_info);

      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &aux_image_memory_barriers,
      );

      device.cmd_copy_image2(transfer_command_buffer, &copy_image_to_aux_info);

      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::BY_REGION,
        &[],
        &[],
        &after_flush_image_memory_barriers,
      );
    }

    self.read_view_index = (self.read_view_index + 1) % crate::consts::SUB_DYNAMIC_BUFFER_COUNT;
  }

  pub(super) fn drop(mut self) {
    let device = self.device.get();

    unsafe {
      self
        .views
        .into_iter()
        .for_each(|view| device.destroy_image_view(view, None));

      self
        .vk_allocator
        .destroy_image(self.image, &mut self.vk_alloc);

      self
        .vk_allocator
        .destroy_buffer(self.staging_buffer, &mut self.staging_vk_alloc);
    }
  }
}
