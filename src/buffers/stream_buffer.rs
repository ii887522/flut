use ash::{
  Device,
  vk::{
    BindBufferMemoryInfo, Buffer, BufferCreateInfo, BufferMemoryRequirementsInfo2,
    BufferUsageFlags, MemoryRequirements2, SharingMode,
  },
};
use gpu_allocator::{
  MemoryLocation,
  vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

pub(crate) struct StreamBuffer {
  device: Arc<Device>,
  pub(crate) buffer: Buffer,
  pub(crate) alloc: Allocation,
}

impl StreamBuffer {
  pub(crate) fn new(
    device: Arc<Device>,
    memory_allocator: Rc<RefCell<Allocator>>,
    name: &str,
    size: u64,
    usage: BufferUsageFlags,
  ) -> Self {
    let buffer_create_info = BufferCreateInfo {
      size,
      usage,
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
        location: MemoryLocation::CpuToGpu,
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

    unsafe {
      device.bind_buffer_memory2(&[bind_buffer_mem_info]).unwrap();
    }

    Self {
      device,
      buffer,
      alloc,
    }
  }
}

impl Drop for StreamBuffer {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_buffer(self.buffer, None);
    }
  }
}
