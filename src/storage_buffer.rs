use crate::{consts, models::range::Range};
use ash::vk;
use std::{ffi::c_void, mem, ptr};
use vk_mem::Alloc as _;

struct StagingBuffer {
  transfer_command_pool: vk::CommandPool,
  transfer_command_buffer: vk::CommandBuffer,
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
  size: usize,
  read_index: usize,
}

impl StorageBuffer {
  pub(super) fn new(
    vk_device: &ash::Device,
    vk_allocator: &vk_mem::Allocator,
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
    size: usize,
  ) -> Self {
    let queue_family_indices = [graphics_queue_family_index, transfer_queue_family_index];

    let (sharing_mode, queue_family_indices) =
      if graphics_queue_family_index == transfer_queue_family_index {
        (vk::SharingMode::EXCLUSIVE, [].as_slice())
      } else {
        (vk::SharingMode::CONCURRENT, queue_family_indices.as_slice())
      };

    let buffer_create_info = vk::BufferCreateInfo {
      size: (consts::MAX_IN_FLIGHT_FRAME_COUNT * size) as u64,
      usage: vk::BufferUsageFlags::STORAGE_BUFFER
        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
        | vk::BufferUsageFlags::TRANSFER_DST,
      sharing_mode,
      queue_family_index_count: queue_family_indices.len().try_into().unwrap(),
      p_queue_family_indices: queue_family_indices.as_ptr(),
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
      let transfer_command_pool_create_info = vk::CommandPoolCreateInfo {
        flags: vk::CommandPoolCreateFlags::TRANSIENT,
        queue_family_index: transfer_queue_family_index,
        ..Default::default()
      };

      let transfer_command_pool = unsafe {
        vk_device
          .create_command_pool(&transfer_command_pool_create_info, None)
          .unwrap()
      };

      let transfer_command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
        command_pool: transfer_command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: 1,
        ..Default::default()
      };

      let transfer_command_buffer = unsafe {
        vk_device
          .allocate_command_buffers(&transfer_command_buffer_alloc_info)
          .unwrap()[0]
      };

      let staging_buffer_create_info = vk::BufferCreateInfo {
        size: (consts::MAX_IN_FLIGHT_FRAME_COUNT * size) as u64,
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
        transfer_command_pool,
        transfer_command_buffer,
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
      size,
      read_index: 0,
    }
  }

  #[inline]
  pub(super) const fn get_read_addr(&self) -> vk::DeviceAddress {
    self.addr + (self.read_index * self.size) as vk::DeviceAddress
  }

  pub(super) fn write<T>(
    &mut self,
    vk_device: &ash::Device,
    items: &[T],
    regions: &[Range],
  ) -> Option<vk::CommandBuffer> {
    let write_index = (self.read_index + 1) % consts::MAX_IN_FLIGHT_FRAME_COUNT;

    let data = match self.data {
      BufferData::Device(data) => data,
      BufferData::Staging(ref staging_buffer) => staging_buffer.data,
    };

    for &Range { start, end } in regions {
      let (start, end) = (start as usize, end as usize);
      let src = items[start..end].as_ptr();

      let dst = unsafe {
        data
          .byte_add(write_index * self.size + start * mem::size_of::<T>())
          .cast()
      };

      unsafe {
        ptr::copy_nonoverlapping(src, dst, end - start);
      }
    }

    let transfer_command_buffer = if !regions.is_empty()
      && let BufferData::Staging(StagingBuffer {
        transfer_command_pool,
        transfer_command_buffer,
        buffer: staging_buffer,
        alloc: _,
        data: _,
      }) = self.data
    {
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

      let regions = regions
        .iter()
        .map(|&Range { start, end }| {
          let (start, end) = (start as usize, end as usize);
          let offset = (write_index * self.size + start * mem::size_of::<T>()) as u64;
          let size = ((end - start) * mem::size_of::<T>()) as u64;

          vk::BufferCopy2 {
            src_offset: offset,
            dst_offset: offset,
            size,
            ..Default::default()
          }
        })
        .collect::<Box<_>>();

      let copy_buffer_info = vk::CopyBufferInfo2 {
        src_buffer: staging_buffer,
        dst_buffer: self.buffer,
        region_count: regions.len().try_into().unwrap(),
        p_regions: regions.as_ptr(),
        ..Default::default()
      };

      unsafe {
        vk_device.cmd_copy_buffer2(transfer_command_buffer, &copy_buffer_info);
      }

      unsafe {
        vk_device
          .end_command_buffer(transfer_command_buffer)
          .unwrap();
      }

      Some(transfer_command_buffer)
    } else {
      None
    };

    self.read_index = write_index;
    transfer_command_buffer
  }

  pub(super) fn drop(mut self, vk_device: &ash::Device, vk_allocator: &vk_mem::Allocator) {
    if let BufferData::Staging(StagingBuffer {
      transfer_command_pool,
      transfer_command_buffer: _,
      buffer: staging_buffer,
      alloc: mut staging_alloc,
      data: _,
    }) = self.data
    {
      unsafe {
        vk_allocator.destroy_buffer(staging_buffer, &mut staging_alloc);
      }
      unsafe {
        vk_device.destroy_command_pool(transfer_command_pool, None);
      }
    }

    unsafe {
      vk_allocator.destroy_buffer(self.buffer, &mut self.alloc);
    }
  }
}
