use super::Device;
use ash::vk;
use gpu_allocator::{
  MemoryLocation,
  vulkan::{self, AllocationScheme},
};
use std::rc::Rc;

pub(super) struct StreamBuffer {
  device: Rc<Device>,
  buffer: vk::Buffer,
  _alloc: vulkan::Allocation,
  addr: vk::DeviceAddress,
}

impl StreamBuffer {
  pub(super) fn new(
    vk_device: Rc<Device>,
    vk_allocator: &mut vulkan::Allocator,
    pageable_device_local_memory_device: &ash::ext::pageable_device_local_memory::Device,
  ) -> Self {
    let device = vk_device.get();

    let buffer_create_info = vk::BufferCreateInfo {
      size: 1024, // todo: Tweak the size which also means capacity in this context
      usage: vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None).unwrap() };
    let mut mem_req = vk::MemoryRequirements2::default();

    let mem_req_info = vk::BufferMemoryRequirementsInfo2 {
      buffer,
      ..Default::default()
    };

    unsafe { device.get_buffer_memory_requirements2(&mem_req_info, &mut mem_req) };

    let alloc_create_desc = vulkan::AllocationCreateDesc {
      name: "stream_buffer",
      requirements: mem_req.memory_requirements,
      location: MemoryLocation::CpuToGpu,
      linear: true,
      allocation_scheme: AllocationScheme::DedicatedBuffer(buffer),
    };

    let alloc = vk_allocator.allocate(&alloc_create_desc).unwrap();

    unsafe {
      (pageable_device_local_memory_device
        .fp()
        .set_device_memory_priority_ext)(device.handle(), alloc.memory(), 1.0);
    };

    let bind_buffer_mem_info = vk::BindBufferMemoryInfo {
      buffer,
      memory: unsafe { alloc.memory() },
      memory_offset: alloc.offset(),
      ..Default::default()
    };

    unsafe { device.bind_buffer_memory2(&[bind_buffer_mem_info]).unwrap() };

    let buffer_device_address_info = vk::BufferDeviceAddressInfo {
      buffer,
      ..Default::default()
    };

    let addr = unsafe { device.get_buffer_device_address(&buffer_device_address_info) };

    Self {
      device: vk_device,
      buffer,
      _alloc: alloc,
      addr,
    }
  }

  pub(super) const fn get_addr(&self) -> vk::DeviceAddress {
    self.addr
  }
}

impl Drop for StreamBuffer {
  fn drop(&mut self) {
    let device = self.device.get();
    unsafe { device.destroy_buffer(self.buffer, None) };
  }
}
