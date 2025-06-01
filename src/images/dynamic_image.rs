use crate::consts;
use ash::{
  Device,
  vk::{
    BindBufferMemoryInfo, BindImageMemoryInfo, Buffer, BufferCreateInfo, BufferImageCopy,
    BufferMemoryRequirementsInfo2, BufferUsageFlags, CommandBuffer, CommandBufferAllocateInfo,
    CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags, CommandPool, Extent3D,
    Fence, FenceCreateFlags, FenceCreateInfo, Format, Image, ImageAspectFlags, ImageCreateInfo,
    ImageLayout, ImageMemoryRequirementsInfo2, ImageSubresourceRange, ImageTiling, ImageType,
    ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, MemoryRequirements2,
    PipelineStageFlags, Queue, SampleCountFlags, Semaphore, SemaphoreCreateInfo, SharingMode,
    SubmitInfo,
  },
};
use gpu_allocator::{
  MemoryLocation,
  vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use rayon::prelude::*;
use std::{cell::RefCell, collections::HashMap, ptr, rc::Rc, sync::Arc};

pub(crate) struct DynamicImage {
  device: Arc<Device>,
  staging_buffers: Vec<Buffer>,
  staging_allocs: Vec<Allocation>,
  pub(crate) images: Vec<Image>,
  _allocs: Vec<Allocation>,
  pub(crate) views: Vec<ImageView>,
  transfer_command_buffers: Vec<CommandBuffer>,
  render_done_semaphore_to_image_index: HashMap<Semaphore, usize>,
  draw_done_semaphores: Vec<Semaphore>,
  fences: Vec<Fence>,
  pub(crate) image_index: usize,
  old_layout: ImageLayout,
  old_pixels: Vec<u8>,
  old_regions: Vec<BufferImageCopy>,
}

impl DynamicImage {
  pub(crate) fn new(
    device: Arc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    transfer_command_pool: CommandPool,
    name: &str,
    format: Format,
    buffer_size: u64,
    image_size: (u32, u32),
    usage: ImageUsageFlags,
  ) -> Self {
    let (staging_buffers, (staging_allocs, bind_staging_buffer_mem_infos)): (
      Vec<_>,
      (Vec<_>, Vec<_>),
    ) = (0..consts::DYNAMIC_IMAGE_COUNT)
      .map(|_| {
        let staging_buffer_create_info = BufferCreateInfo {
          size: buffer_size,
          usage: BufferUsageFlags::TRANSFER_SRC,
          sharing_mode: SharingMode::EXCLUSIVE,
          ..Default::default()
        };

        let staging_buffer = unsafe {
          device
            .create_buffer(&staging_buffer_create_info, None)
            .unwrap()
        };

        let staging_buffer_mem_req_info = BufferMemoryRequirementsInfo2 {
          buffer: staging_buffer,
          ..Default::default()
        };

        let mut staging_buffer_mem_req = MemoryRequirements2::default();

        unsafe {
          device.get_buffer_memory_requirements2(
            &staging_buffer_mem_req_info,
            &mut staging_buffer_mem_req,
          );
        }

        let staging_alloc = memory_allocator
          .borrow_mut()
          .allocate(&AllocationCreateDesc {
            name: &format!("staging_{name}"),
            requirements: staging_buffer_mem_req.memory_requirements,
            location: MemoryLocation::CpuToGpu,
            linear: true,
            allocation_scheme: AllocationScheme::DedicatedBuffer(staging_buffer),
          })
          .unwrap();

        let bind_staging_buffer_mem_info = BindBufferMemoryInfo {
          buffer: staging_buffer,
          memory: unsafe { staging_alloc.memory() },
          memory_offset: staging_alloc.offset(),
          ..Default::default()
        };

        (
          staging_buffer,
          (staging_alloc, bind_staging_buffer_mem_info),
        )
      })
      .unzip();

    let (images, (allocs, bind_image_mem_infos)): (Vec<_>, (Vec<_>, Vec<_>)) = (0
      ..consts::DYNAMIC_IMAGE_COUNT)
      .map(|_| {
        let image_create_info = ImageCreateInfo {
          image_type: ImageType::TYPE_2D,
          format,
          extent: Extent3D {
            width: image_size.0,
            height: image_size.1,
            depth: 1,
          },
          mip_levels: 1,
          array_layers: 1,
          samples: SampleCountFlags::TYPE_1,
          tiling: ImageTiling::OPTIMAL,
          usage: ImageUsageFlags::TRANSFER_DST | usage,
          sharing_mode: SharingMode::EXCLUSIVE,
          initial_layout: ImageLayout::UNDEFINED,
          ..Default::default()
        };

        let image = unsafe { device.create_image(&image_create_info, None).unwrap() };

        let image_mem_req_info = ImageMemoryRequirementsInfo2 {
          image,
          ..Default::default()
        };

        let mut image_mem_req = MemoryRequirements2::default();

        unsafe {
          device.get_image_memory_requirements2(&image_mem_req_info, &mut image_mem_req);
        }

        let alloc = memory_allocator
          .borrow_mut()
          .allocate(&AllocationCreateDesc {
            name,
            requirements: image_mem_req.memory_requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::DedicatedImage(image),
          })
          .unwrap();

        let bind_image_mem_info = BindImageMemoryInfo {
          image,
          memory: unsafe { alloc.memory() },
          memory_offset: alloc.offset(),
          ..Default::default()
        };

        (image, (alloc, bind_image_mem_info))
      })
      .unzip();

    let (draw_done_semaphores, fences): (Vec<_>, Vec<_>) = (0..consts::DYNAMIC_IMAGE_COUNT)
      .map(|_| {
        let draw_done_semaphore_create_info = SemaphoreCreateInfo::default();

        let fence_create_info = FenceCreateInfo {
          flags: FenceCreateFlags::SIGNALED,
          ..Default::default()
        };

        unsafe {
          (
            device
              .create_semaphore(&draw_done_semaphore_create_info, None)
              .unwrap(),
            device.create_fence(&fence_create_info, None).unwrap(),
          )
        }
      })
      .unzip();

    let transfer_command_buffer_alloc_info = CommandBufferAllocateInfo {
      command_pool: transfer_command_pool,
      level: CommandBufferLevel::PRIMARY,
      command_buffer_count: consts::DYNAMIC_IMAGE_COUNT as _,
      ..Default::default()
    };

    let transfer_command_buffers = unsafe {
      device
        .allocate_command_buffers(&transfer_command_buffer_alloc_info)
        .unwrap()
    };

    unsafe {
      device
        .bind_buffer_memory2(&bind_staging_buffer_mem_infos)
        .unwrap();

      device.bind_image_memory2(&bind_image_mem_infos).unwrap();
    }

    let views = images
      .iter()
      .map(|&image| {
        let image_view_create_info = ImageViewCreateInfo {
          image,
          view_type: ImageViewType::TYPE_2D,
          format,
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
          device
            .create_image_view(&image_view_create_info, None)
            .unwrap()
        }
      })
      .collect::<Vec<_>>();

    Self {
      device,
      staging_buffers,
      staging_allocs,
      images,
      _allocs: allocs,
      views,
      transfer_command_buffers,
      render_done_semaphore_to_image_index: HashMap::new(),
      draw_done_semaphores,
      fences,
      image_index: 0,
      old_layout: ImageLayout::UNDEFINED,
      old_pixels: vec![],
      old_regions: vec![],
    }
  }

  pub(crate) fn get_draw_done_semaphore(&self) -> Semaphore {
    self.draw_done_semaphores[self.image_index]
  }

  pub(crate) fn set_render_done_semaphore(&mut self, semaphore: Semaphore) {
    self
      .render_done_semaphore_to_image_index
      .insert(semaphore, self.image_index);
  }

  pub(crate) fn draw(
    &mut self,
    transfer_queue: Queue,
    mut pixels: Vec<u8>,
    mut regions: Vec<BufferImageCopy>,
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
  ) {
    unsafe {
      self.image_index = (self.image_index + 1) % consts::DYNAMIC_IMAGE_COUNT as usize;

      let transfer_command_buffer = self.transfer_command_buffers[self.image_index];
      let draw_done_semaphore = self.draw_done_semaphores[self.image_index];
      let fence = self.fences[self.image_index];
      let staging_buffer = self.staging_buffers[self.image_index];
      let image = self.images[self.image_index];
      let mapped_staging_alloc = self.staging_allocs[self.image_index].mapped_ptr().unwrap();
      let new_region_count = regions.len();
      let new_pixel_count = pixels.len();

      regions.par_extend(
        self
          .old_regions
          .par_drain(..)
          .map(|region| BufferImageCopy {
            buffer_offset: region.buffer_offset + pixels.len() as u64,
            ..region
          }),
      );

      pixels.par_extend(self.old_pixels.par_drain(..));

      self
        .device
        .wait_for_fences(&[fence], true, u64::MAX)
        .unwrap();

      ptr::copy_nonoverlapping(
        pixels.as_ptr(),
        mapped_staging_alloc.as_ptr() as *mut _,
        pixels.len(),
      );

      let transfer_command_buffer_begin_info = CommandBufferBeginInfo {
        flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
      };

      self
        .device
        .begin_command_buffer(transfer_command_buffer, &transfer_command_buffer_begin_info)
        .unwrap();

      crate::record_copy_buffer_to_image_commands(
        self.device.clone(),
        transfer_command_buffer,
        staging_buffer,
        image,
        &regions,
        graphics_queue_family_index,
        transfer_queue_family_index,
        self.old_layout,
      );

      self
        .device
        .end_command_buffer(transfer_command_buffer)
        .unwrap();

      self.device.reset_fences(&[fence]).unwrap();

      let (render_done_semaphores, wait_dst_stage_mask): (Vec<_>, Vec<_>) = self
        .render_done_semaphore_to_image_index
        .iter()
        .filter_map(|(&render_done_semaphore, &image_index)| {
          if image_index == self.image_index {
            Some((render_done_semaphore, PipelineStageFlags::TRANSFER))
          } else {
            None
          }
        })
        .unzip();

      let queue_submit_info = SubmitInfo {
        wait_semaphore_count: render_done_semaphores.len() as _,
        p_wait_semaphores: render_done_semaphores.as_ptr(),
        p_wait_dst_stage_mask: wait_dst_stage_mask.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &transfer_command_buffer,
        signal_semaphore_count: 1,
        p_signal_semaphores: &draw_done_semaphore,
        ..Default::default()
      };

      self
        .device
        .queue_submit(transfer_queue, &[queue_submit_info], fence)
        .unwrap();

      pixels.truncate(new_pixel_count);
      regions.truncate(new_region_count);
      self.old_layout = ImageLayout::SHADER_READ_ONLY_OPTIMAL;
      self.old_pixels = pixels;
      self.old_regions = regions;
    }
  }
}

impl Drop for DynamicImage {
  fn drop(&mut self) {
    unsafe {
      self.fences.iter().for_each(|&fence| {
        self.device.destroy_fence(fence, None);
      });

      self.draw_done_semaphores.iter().for_each(|&semaphore| {
        self.device.destroy_semaphore(semaphore, None);
      });

      self.views.iter().for_each(|&view| {
        self.device.destroy_image_view(view, None);
      });

      self.images.iter().for_each(|&image| {
        self.device.destroy_image(image, None);
      });

      self.staging_buffers.iter().for_each(|&staging_buffer| {
        self.device.destroy_buffer(staging_buffer, None);
      });
    };
  }
}
