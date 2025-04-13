use ash::{
  Device,
  vk::{
    BindBufferMemoryInfo, BindImageMemoryInfo, Buffer, BufferCreateInfo,
    BufferMemoryRequirementsInfo2, BufferUsageFlags, Extent3D, Format, Image, ImageAspectFlags,
    ImageCreateInfo, ImageLayout, ImageMemoryRequirementsInfo2, ImageSubresourceRange, ImageTiling,
    ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, MemoryRequirements2,
    SampleCountFlags, SharingMode,
  },
};
use gpu_allocator::{
  MemoryLocation,
  vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::{cell::RefCell, mem, ptr, rc::Rc};

pub(crate) struct StaticImage {
  device: Rc<Device>,
  memory_allocator: Rc<RefCell<Allocator>>,
  pub(crate) staging_buffer: Buffer,
  staging_alloc: Allocation,
  pub(crate) image: Image,
  _alloc: Allocation,
  pub(crate) view: ImageView,
}

impl StaticImage {
  pub(crate) fn new(
    device: Rc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    name: &str,
    format: Format,
    size: (u32, u32),
    usage: ImageUsageFlags,
    data: &[u8],
  ) -> Self {
    let staging_buffer_create_info = BufferCreateInfo {
      size: mem::size_of_val(data) as _,
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
      device
        .get_buffer_memory_requirements2(&staging_buffer_mem_req_info, &mut staging_buffer_mem_req);
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

    let mapped_staging_alloc = staging_alloc.mapped_ptr().unwrap();

    unsafe {
      ptr::copy_nonoverlapping(
        data.as_ptr(),
        mapped_staging_alloc.as_ptr() as *mut _,
        data.len(),
      );
    }

    let image_create_info = ImageCreateInfo {
      image_type: ImageType::TYPE_2D,
      format,
      extent: Extent3D {
        width: size.0,
        height: size.1,
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

    unsafe {
      device
        .bind_buffer_memory2(&[bind_staging_buffer_mem_info])
        .unwrap();

      device.bind_image_memory2(&[bind_image_mem_info]).unwrap();
    }

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

    let image_view = unsafe {
      device
        .create_image_view(&image_view_create_info, None)
        .unwrap()
    };

    Self {
      device,
      memory_allocator,
      staging_buffer,
      staging_alloc,
      image,
      _alloc: alloc,
      view: image_view,
    }
  }

  pub(crate) fn drop_staging(&mut self) {
    unsafe {
      self.device.destroy_buffer(self.staging_buffer, None);
    }

    self
      .memory_allocator
      .borrow_mut()
      .free(mem::take(&mut self.staging_alloc))
      .unwrap();
  }
}

impl Drop for StaticImage {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_image_view(self.view, None);
      self.device.destroy_image(self.image, None);
    };
  }
}
