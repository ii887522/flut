use super::Device;
use ash::vk;
use std::{ffi::c_void, rc::Rc};
use vk_mem::Alloc;

pub(super) struct StreamBuffer {
  vk_allocator: Rc<vk_mem::Allocator>,
  buffer: vk::Buffer,
  vk_alloc: vk_mem::Allocation,
  mapped_data: *mut c_void,
  addr: vk::DeviceAddress,
  max_bytes: usize,
  sub_buf_index: usize,
}

impl StreamBuffer {
  pub(super) fn new(
    vk_device: Rc<Device>,
    vk_allocator: Rc<vk_mem::Allocator>,
    max_bytes: usize,
  ) -> Self {
    let device = vk_device.get();

    let buffer_create_info = vk::BufferCreateInfo {
      size: (max_bytes * crate::consts::MAX_IN_FLIGHT_FRAME_COUNT) as _,
      usage: vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let alloc_create_info = vk_mem::AllocationCreateInfo {
      flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
        | vk_mem::AllocationCreateFlags::MAPPED,
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      priority: 1.0,
      ..Default::default()
    };

    let (buffer, vk_alloc) = unsafe {
      vk_allocator
        .create_buffer(&buffer_create_info, &alloc_create_info)
        .unwrap()
    };

    let alloc_info = vk_allocator.get_allocation_info(&vk_alloc);

    let buffer_device_address_info = vk::BufferDeviceAddressInfo {
      buffer,
      ..Default::default()
    };

    let addr = unsafe { device.get_buffer_device_address(&buffer_device_address_info) };

    Self {
      vk_allocator,
      buffer,
      vk_alloc,
      mapped_data: alloc_info.mapped_data,
      addr,
      max_bytes,
      sub_buf_index: 0,
    }
  }

  pub(super) const fn get_addr(&self) -> vk::DeviceAddress {
    self.addr + (self.sub_buf_index * self.max_bytes) as u64
  }

  pub(super) const fn get_mapped_data(&self) -> *mut c_void {
    unsafe {
      self
        .mapped_data
        .byte_add(self.sub_buf_index * self.max_bytes)
    }
  }

  pub(super) fn next_sub_buf(self) -> Self {
    Self {
      sub_buf_index: (self.sub_buf_index + 1) % crate::consts::MAX_IN_FLIGHT_FRAME_COUNT,
      ..self
    }
  }

  pub(super) fn drop(mut self) {
    unsafe {
      self
        .vk_allocator
        .destroy_buffer(self.buffer, &mut self.vk_alloc);
    }
  }
}
