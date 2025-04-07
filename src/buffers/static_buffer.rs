use ash::{
  Device,
  vk::{
    BindBufferMemoryInfo, Buffer, BufferCopy, BufferCreateInfo, BufferMemoryRequirementsInfo2,
    BufferUsageFlags, MemoryRequirements2, SharingMode,
  },
};
use gpu_allocator::{
  MemoryLocation,
  vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::{cell::RefCell, mem, ptr, rc::Rc};

pub(crate) struct StaticBuffer<'a> {
  device: Rc<Device>,
  memory_allocator: Rc<RefCell<Allocator>>,
  pub(crate) staging_buffer: Buffer,
  staging_alloc: Allocation,
  pub(crate) buffer: Buffer,
  _alloc: Allocation,
  pub(crate) bind_staging_buffer_mem_info: BindBufferMemoryInfo<'a>,
  pub(crate) bind_buffer_mem_info: BindBufferMemoryInfo<'a>,
  pub(crate) buffer_copy: BufferCopy,
}

impl StaticBuffer<'_> {
  pub(crate) fn new<T: Copy>(
    device: Rc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    name: &str,
    usage: BufferUsageFlags,
    data: &[T],
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
        mapped_staging_alloc.as_ptr() as *mut T,
        data.len(),
      );
    }

    let buffer_create_info = BufferCreateInfo {
      size: mem::size_of_val(data) as _,
      usage: BufferUsageFlags::TRANSFER_DST | usage,
      sharing_mode: SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None).unwrap() };

    let buffer_mem_req_info = BufferMemoryRequirementsInfo2 {
      buffer,
      ..Default::default()
    };

    let mut buffer_mem_req = MemoryRequirements2::default();

    unsafe {
      device.get_buffer_memory_requirements2(&buffer_mem_req_info, &mut buffer_mem_req);
    }

    let alloc = memory_allocator
      .borrow_mut()
      .allocate(&AllocationCreateDesc {
        name,
        requirements: buffer_mem_req.memory_requirements,
        location: MemoryLocation::GpuOnly,
        linear: true,
        allocation_scheme: AllocationScheme::DedicatedBuffer(buffer),
      })
      .unwrap();

    let bind_buffer_mem_info = BindBufferMemoryInfo {
      buffer,
      memory: unsafe { alloc.memory() },
      memory_offset: alloc.offset(),
      ..Default::default()
    };

    let buffer_copy = BufferCopy {
      size: mem::size_of_val(data) as _,
      ..Default::default()
    };

    Self {
      device,
      memory_allocator,
      staging_buffer,
      staging_alloc,
      buffer,
      _alloc: alloc,
      bind_staging_buffer_mem_info,
      bind_buffer_mem_info,
      buffer_copy,
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

impl Drop for StaticBuffer<'_> {
  fn drop(&mut self) {
    unsafe { self.device.destroy_buffer(self.buffer, None) };
  }
}
