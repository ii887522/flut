use crate::consts;
use ash::vk;
use std::ffi::c_void;
use vk_mem::Alloc as _;

struct StagingBuffer {
  buffer: vk::Buffer,
  alloc: vk_mem::Allocation,
  data: *mut c_void,
}

enum BufferData {
  Device(*mut c_void),
  Staging(StagingBuffer),
}

pub struct StorageBuffer {
  buffer: vk::Buffer,
  alloc: vk_mem::Allocation,
  addr: vk::DeviceAddress,
  data: BufferData,
}

impl StorageBuffer {
  pub(super) fn new(vk_device: &ash::Device, vk_allocator: &vk_mem::Allocator) -> Self {
    let buffer_create_info = vk::BufferCreateInfo {
      size: (consts::MAX_IN_FLIGHT_FRAME_COUNT * 1024 * 1024) as u64, // 2MB
      usage: vk::BufferUsageFlags::STORAGE_BUFFER
        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
        | vk::BufferUsageFlags::TRANSFER_DST,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let alloc_create_info = vk_mem::AllocationCreateInfo {
      flags: vk_mem::AllocationCreateFlags::MAPPED
        | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
        | vk_mem::AllocationCreateFlags::HOST_ACCESS_ALLOW_TRANSFER_INSTEAD,
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      priority: 1.0,
      ..Default::default()
    };

    let (buffer, alloc) = unsafe {
      vk_allocator
        .create_buffer(&buffer_create_info, &alloc_create_info)
        .unwrap()
    };

    let buffer_device_address_info = vk::BufferDeviceAddressInfo {
      buffer,
      ..Default::default()
    };

    let addr = unsafe { vk_device.get_buffer_device_address(&buffer_device_address_info) };
    let alloc_info = vk_allocator.get_allocation_info2(&alloc);
    let data = alloc_info.allocation_info.mapped_data;

    let buffer_data = if data.is_null() {
      let staging_buffer_create_info = vk::BufferCreateInfo {
        size: ((consts::MAX_IN_FLIGHT_FRAME_COUNT - 1) * 1024 * 1024) as u64, // 1MB
        usage: vk::BufferUsageFlags::TRANSFER_SRC,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
      };

      let staging_alloc_create_info = vk_mem::AllocationCreateInfo {
        flags: vk_mem::AllocationCreateFlags::MAPPED
          | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
        usage: vk_mem::MemoryUsage::AutoPreferHost,
        priority: 1.0,
        ..Default::default()
      };

      let (staging_buffer, staging_alloc) = unsafe {
        vk_allocator
          .create_buffer(&staging_buffer_create_info, &staging_alloc_create_info)
          .unwrap()
      };

      let staging_alloc_info = vk_allocator.get_allocation_info2(&staging_alloc);
      let staging_data = staging_alloc_info.allocation_info.mapped_data;

      BufferData::Staging(StagingBuffer {
        buffer: staging_buffer,
        alloc: staging_alloc,
        data: staging_data,
      })
    } else {
      BufferData::Device(data)
    };

    Self {
      buffer,
      alloc,
      addr,
      data: buffer_data,
    }
  }

  #[inline]
  pub(super) const fn get_addr(&self) -> vk::DeviceAddress {
    self.addr
  }

  pub(super) fn drop(mut self, vk_allocator: &vk_mem::Allocator) {
    if let BufferData::Staging(mut staging_buffer) = self.data {
      unsafe {
        vk_allocator.destroy_buffer(staging_buffer.buffer, &mut staging_buffer.alloc);
      }
    }

    unsafe {
      vk_allocator.destroy_buffer(self.buffer, &mut self.alloc);
    }
  }
}
