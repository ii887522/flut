use super::Device;
use ash::vk;
use std::rc::Rc;
use vk_mem::Alloc;

pub(super) struct StreamBuffer {
  vk_allocator: Rc<vk_mem::Allocator>,
  buffer: vk::Buffer,
  vk_alloc: vk_mem::Allocation,
  addr: vk::DeviceAddress,
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
      flags: vk_mem::AllocationCreateFlags::MAPPED
        | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      priority: 1.0,
      ..Default::default()
    };

    let (buffer, vk_alloc) = unsafe {
      vk_allocator
        .create_buffer(&buffer_create_info, &alloc_create_info)
        .unwrap()
    };

    let buffer_device_address_info = vk::BufferDeviceAddressInfo {
      buffer,
      ..Default::default()
    };

    let addr = unsafe { device.get_buffer_device_address(&buffer_device_address_info) };

    Self {
      vk_allocator,
      buffer,
      vk_alloc,
      addr,
    }
  }

  pub(super) const fn get_addr(&self) -> vk::DeviceAddress {
    self.addr
  }
}

impl Drop for StreamBuffer {
  fn drop(&mut self) {
    unsafe {
      self
        .vk_allocator
        .destroy_buffer(self.buffer, &mut self.vk_alloc);
    }
  }
}
