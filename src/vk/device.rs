use ash::vk;
use rustc_hash::FxHashSet;

pub(super) struct Device {
  device: ash::Device,
  pipeline_creation_cache_control: bool,
}

impl Device {
  pub(super) fn new(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    graphics_queue_family_index: u32,
    present_queue_family_index: u32,
  ) -> Self {
    let queue_priorities = [1.0];

    let queue_create_infos =
      FxHashSet::from_iter([graphics_queue_family_index, present_queue_family_index])
        .into_iter()
        .map(|queue_family_index| vk::DeviceQueueCreateInfo {
          queue_family_index,
          queue_count: queue_priorities.len() as _,
          p_queue_priorities: queue_priorities.as_ptr(),
          ..Default::default()
        })
        .collect::<Box<_>>();

    let ext_props_list = unsafe {
      instance
        .enumerate_device_extension_properties(physical_device)
        .unwrap()
    };

    let has_portability_subset_ext = ext_props_list.into_iter().any(|ext_props| {
      let ext_name = ext_props.extension_name_as_c_str().unwrap();
      ext_name == vk::KHR_PORTABILITY_SUBSET_NAME
    });

    let ext_names = [
      vk::EXT_MEMORY_PRIORITY_NAME.as_ptr(),
      vk::EXT_PAGEABLE_DEVICE_LOCAL_MEMORY_NAME.as_ptr(),
      vk::KHR_SWAPCHAIN_NAME.as_ptr(),
    ];

    let ext_names_with_portability_subset = [
      vk::EXT_MEMORY_PRIORITY_NAME.as_ptr(),
      vk::EXT_PAGEABLE_DEVICE_LOCAL_MEMORY_NAME.as_ptr(),
      vk::KHR_SWAPCHAIN_NAME.as_ptr(),
      vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr(),
    ];

    let ext_names = if has_portability_subset_ext {
      ext_names_with_portability_subset.as_slice()
    } else {
      ext_names.as_slice()
    };

    let mut physical_device_pipeline_creation_cache_control_features =
      vk::PhysicalDevicePipelineCreationCacheControlFeatures::default();

    let mut physical_device_features = vk::PhysicalDeviceFeatures2 {
      p_next: &mut physical_device_pipeline_creation_cache_control_features as *mut _ as *mut _,
      ..Default::default()
    };

    unsafe {
      instance.get_physical_device_features2(physical_device, &mut physical_device_features)
    };

    let physical_device_8bit_storage_features = vk::PhysicalDevice8BitStorageFeatures {
      uniform_and_storage_buffer8_bit_access: vk::TRUE,
      p_next: &physical_device_pipeline_creation_cache_control_features as *const _ as *mut _,
      ..Default::default()
    };

    let physical_device_buffer_device_address_features =
      vk::PhysicalDeviceBufferDeviceAddressFeatures {
        buffer_device_address: vk::TRUE,
        p_next: &physical_device_8bit_storage_features as *const _ as *mut _,
        ..Default::default()
      };

    let physical_device_vulkan_memory_model_features =
      vk::PhysicalDeviceVulkanMemoryModelFeatures {
        vulkan_memory_model: vk::TRUE,
        vulkan_memory_model_device_scope: vk::TRUE,
        p_next: &physical_device_buffer_device_address_features as *const _ as *mut _,
        ..Default::default()
      };

    let physical_device_timeline_semaphore_features = vk::PhysicalDeviceTimelineSemaphoreFeatures {
      timeline_semaphore: vk::TRUE,
      p_next: &physical_device_vulkan_memory_model_features as *const _ as *mut _,
      ..Default::default()
    };

    let physical_device_pageable_device_local_memory_features =
      vk::PhysicalDevicePageableDeviceLocalMemoryFeaturesEXT {
        pageable_device_local_memory: vk::TRUE,
        p_next: &physical_device_timeline_semaphore_features as *const _ as *mut _,
        ..Default::default()
      };

    let physical_device_features = vk::PhysicalDeviceFeatures2 {
      features: vk::PhysicalDeviceFeatures {
        fragment_stores_and_atomics: vk::TRUE,
        vertex_pipeline_stores_and_atomics: vk::TRUE,
        shader_int64: vk::TRUE,
        ..Default::default()
      },
      p_next: &physical_device_pageable_device_local_memory_features as *const _ as *mut _,
      ..Default::default()
    };

    let device_create_info = vk::DeviceCreateInfo {
      queue_create_info_count: queue_create_infos.len() as _,
      p_queue_create_infos: queue_create_infos.as_ptr(),
      enabled_extension_count: ext_names.len() as _,
      pp_enabled_extension_names: ext_names.as_ptr(),
      p_next: &physical_device_features as *const _ as *const _,
      ..Default::default()
    };

    let device = unsafe {
      instance
        .create_device(physical_device, &device_create_info, None)
        .unwrap()
    };

    let pipeline_creation_cache_control = physical_device_pipeline_creation_cache_control_features
      .pipeline_creation_cache_control
      == vk::TRUE;

    Self {
      device,
      pipeline_creation_cache_control,
    }
  }

  pub(super) const fn get(&self) -> &ash::Device {
    &self.device
  }

  pub(super) const fn pipeline_creation_cache_control(&self) -> bool {
    self.pipeline_creation_cache_control
  }
}
