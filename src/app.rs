use ash::vk;
use std::ffi::{CStr, CString, c_char};
use winit::{
  application::ApplicationHandler,
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
  event_loop::{ActiveEventLoop, EventLoop},
  window::{Window, WindowButtons},
};

const VK_INSTANCE_EXT_NAMES: &[*const c_char] = &[
  vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr(),
  #[cfg(debug_assertions)]
  vk::EXT_LAYER_SETTINGS_NAME.as_ptr(),
];

const VK_DEVICE_EXT_NAMES: &[&CStr] = &[
  vk::KHR_PORTABILITY_SUBSET_NAME,
  vk::EXT_PAGEABLE_DEVICE_LOCAL_MEMORY_NAME,
  vk::EXT_MEMORY_PRIORITY_NAME,
];

pub fn run<App: ApplicationHandler>(mut app: App) {
  let event_loop = match EventLoop::new() {
    Ok(event_loop) => event_loop,
    Err(err) => panic!("{err}"),
  };

  if let Err(err) = event_loop.run_app(&mut app) {
    panic!("{err}");
  }
}

pub struct App {
  window: Window,
  vk_entry: ash::Entry,
  vk_instance: ash::Instance,
  vk_device: ash::Device,
}

impl App {
  #[must_use]
  pub fn new(event_loop: &ActiveEventLoop, title: &str, size: (f64, f64)) -> Self {
    let (width, height) = size;

    let window = match event_loop.create_window(
      Window::default_attributes()
        .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
        .with_inner_size(Size::Logical(LogicalSize::new(width, height)))
        .with_position(Position::Logical(LogicalPosition::new(8_f64, 8_f64)))
        .with_resizable(false)
        .with_title(title)
        .with_visible(false),
    ) {
      Ok(window) => window,
      Err(err) => panic!("{err}"),
    };

    if let Some(current_monitor) = window.current_monitor() {
      let PhysicalSize {
        width: window_outer_width,
        height: window_outer_height,
      } = window.outer_size();

      let PhysicalSize {
        width: monitor_width,
        height: monitor_height,
      } = current_monitor.size();

      let (window_outer_x, window_outer_y) = (
        monitor_width.saturating_sub(window_outer_width) >> 1_u32,
        monitor_height.saturating_sub(window_outer_height) >> 1_u32,
      );

      window.set_outer_position(Position::Physical(PhysicalPosition::new(
        window_outer_x.cast_signed(),
        window_outer_y.cast_signed(),
      )));
    } else {
      eprintln!("Failed to get current monitor to center the window");
    }

    let vk_entry = match unsafe { ash::Entry::load() } {
      Ok(entry) => entry,
      Err(err) => panic!("{err}"),
    };

    let vk_app_info = vk::ApplicationInfo {
      api_version: vk::make_api_version(0, 1, 3, 0),
      ..Default::default()
    };

    #[cfg(debug_assertions)]
    let vk_validation_layer_name = match CString::new("VK_LAYER_KHRONOS_validation") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let validate_sync_name = match CString::new("validate_sync") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let syncval_shader_accesses_heuristic_name =
      match CString::new("syncval_shader_accesses_heuristic") {
        Ok(name) => name,
        Err(err) => panic!("{err}"),
      };

    #[cfg(debug_assertions)]
    let printf_enable_name = match CString::new("printf_enable") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let gpuav_enable_name = match CString::new("gpuav_enable") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let gpuav_validate_ray_query_name = match CString::new("gpuav_validate_ray_query") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let validate_best_practices_name = match CString::new("validate_best_practices") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let validate_best_practices_arm_name = match CString::new("validate_best_practices_arm") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let validate_best_practices_amd_name = match CString::new("validate_best_practices_amd") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let validate_best_practices_img_name = match CString::new("validate_best_practices_img") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let validate_best_practices_nvidia_name = match CString::new("validate_best_practices_nvidia") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let report_flags_name = match CString::new("report_flags") {
      Ok(name) => name,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let validate_sync_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let syncval_shader_accesses_heuristic_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let printf_enable_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let gpuav_enable_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let gpuav_validate_ray_query_value = vk::FALSE;

    #[cfg(debug_assertions)]
    let validate_best_practices_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let validate_best_practices_arm_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let validate_best_practices_amd_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let validate_best_practices_img_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let validate_best_practices_nvidia_value = vk::TRUE;

    #[cfg(debug_assertions)]
    let report_flags_values = [
      match CString::new("info") {
        Ok(value) => value,
        Err(err) => panic!("{err}"),
      },
      match CString::new("warn") {
        Ok(value) => value,
        Err(err) => panic!("{err}"),
      },
      match CString::new("perf") {
        Ok(value) => value,
        Err(err) => panic!("{err}"),
      },
      match CString::new("error") {
        Ok(value) => value,
        Err(err) => panic!("{err}"),
      },
    ];

    #[cfg(debug_assertions)]
    let report_flags_values = report_flags_values
      .iter()
      .map(|value| value.as_ptr())
      .collect::<Box<_>>();

    #[cfg(debug_assertions)]
    let report_flags_value_count = match report_flags_values.len().try_into() {
      Ok(count) => count,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let vk_layer_settings = [
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: validate_sync_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const validate_sync_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: syncval_shader_accesses_heuristic_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const syncval_shader_accesses_heuristic_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: printf_enable_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const printf_enable_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: gpuav_enable_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const gpuav_enable_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: gpuav_validate_ray_query_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const gpuav_validate_ray_query_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const validate_best_practices_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_arm_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const validate_best_practices_arm_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_amd_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const validate_best_practices_amd_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_img_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const validate_best_practices_img_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_nvidia_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: (&raw const validate_best_practices_nvidia_value).cast(),
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: vk_validation_layer_name.as_ptr(),
        p_setting_name: report_flags_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::STRING,
        value_count: report_flags_value_count,
        p_values: report_flags_values.as_ptr().cast(),
        ..Default::default()
      },
    ];

    let vk_layer_names: [CString; _] = [
      #[cfg(debug_assertions)]
      vk_validation_layer_name,
    ];

    let vk_layer_names = vk_layer_names
      .iter()
      .map(|name| name.as_ptr())
      .collect::<Box<_>>();

    let vk_layer_count = match vk_layer_names.len().try_into() {
      Ok(count) => count,
      Err(err) => panic!("{err}"),
    };

    let vk_instance_ext_count = match VK_INSTANCE_EXT_NAMES.len().try_into() {
      Ok(count) => count,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let vk_layer_setting_count = match vk_layer_settings.len().try_into() {
      Ok(count) => count,
      Err(err) => panic!("{err}"),
    };

    #[cfg(debug_assertions)]
    let vk_layer_settings_create_info = vk::LayerSettingsCreateInfoEXT {
      setting_count: vk_layer_setting_count,
      p_settings: vk_layer_settings.as_ptr(),
      ..Default::default()
    };

    let vk_instance_create_info = vk::InstanceCreateInfo {
      flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
      p_application_info: &raw const vk_app_info,
      enabled_layer_count: vk_layer_count,
      pp_enabled_layer_names: vk_layer_names.as_ptr(),
      enabled_extension_count: vk_instance_ext_count,
      pp_enabled_extension_names: VK_INSTANCE_EXT_NAMES.as_ptr(),

      #[cfg(debug_assertions)]
      p_next: (&raw const vk_layer_settings_create_info).cast(),

      ..Default::default()
    };

    let vk_instance = match unsafe { vk_entry.create_instance(&vk_instance_create_info, None) } {
      Ok(instance) => instance,
      Err(err) => panic!("{err}"),
    };

    let vk_physical_devices = match unsafe { vk_instance.enumerate_physical_devices() } {
      Ok(devices) => devices,
      Err(err) => panic!("{err}"),
    };

    let Some((vk_physical_device, graphics_queue_family_index)) = vk_physical_devices
      .into_iter()
      .filter_map(|vk_physical_device| {
        let queue_family_props_count = unsafe {
          vk_instance.get_physical_device_queue_family_properties2_len(vk_physical_device)
        };

        let mut queue_family_props =
          vec![vk::QueueFamilyProperties2::default(); queue_family_props_count];

        unsafe {
          vk_instance.get_physical_device_queue_family_properties2(
            vk_physical_device,
            &mut queue_family_props,
          );
        }

        let queue_family_props = queue_family_props;

        let graphics_queue_family_index = match queue_family_props
          .into_iter()
          .enumerate()
          .find_map(|(index, queue_family_props)| {
            queue_family_props
              .queue_family_properties
              .queue_flags
              .contains(vk::QueueFlags::GRAPHICS)
              .then_some(index)
          })?
          .try_into()
        {
          Ok(index) => index,
          Err(err) => panic!("{err}"),
        };

        Some((vk_physical_device, graphics_queue_family_index))
      })
      .max_by_key(|&(vk_physical_device, _graphics_queue_family_index)| {
        let mut vk_physical_device_props = vk::PhysicalDeviceProperties2::default();

        unsafe {
          vk_instance
            .get_physical_device_properties2(vk_physical_device, &mut vk_physical_device_props);
        }

        let vk_physical_device_props = vk_physical_device_props;

        match vk_physical_device_props.properties.device_type {
          vk::PhysicalDeviceType::INTEGRATED_GPU => 4_u32,
          vk::PhysicalDeviceType::DISCRETE_GPU => 3_u32,
          vk::PhysicalDeviceType::VIRTUAL_GPU => 2_u32,
          vk::PhysicalDeviceType::CPU => 1_u32,
          _ => 0_u32,
        }
      })
    else {
      panic!("No suitable physical device found");
    };

    let queue_priorities = [1.0];

    let queue_count = match queue_priorities.len().try_into() {
      Ok(count) => count,
      Err(err) => panic!("{err}"),
    };

    let vk_queue_create_infos = [vk::DeviceQueueCreateInfo {
      queue_family_index: graphics_queue_family_index,
      queue_count,
      p_queue_priorities: queue_priorities.as_ptr(),
      ..Default::default()
    }];

    let vk_queue_create_info_count = match vk_queue_create_infos.len().try_into() {
      Ok(count) => count,
      Err(err) => panic!("{err}"),
    };

    let vk_ext_props = unsafe {
      vk_instance
        .enumerate_device_extension_properties(vk_physical_device)
        .unwrap_or_else(|err| {
          eprintln!("Failed to enumerate selected physical device extension properties: {err}");
          vec![]
        })
    };

    let vk_device_ext_names = vk_ext_props
      .into_iter()
      .filter_map(|vk_ext_props| {
        let Ok(vk_ext_name) = vk_ext_props.extension_name_as_c_str() else {
          return None;
        };

        VK_DEVICE_EXT_NAMES.iter().find_map(|&req_vk_ext_name| {
          (req_vk_ext_name == vk_ext_name).then_some(req_vk_ext_name.as_ptr())
        })
      })
      .collect::<Box<_>>();

    let vk_device_ext_count = match vk_device_ext_names.len().try_into() {
      Ok(count) => count,
      Err(err) => panic!("{err}"),
    };

    let mut vk_physical_device_8_bit_storage_features = vk::PhysicalDevice8BitStorageFeatures {
      uniform_and_storage_buffer8_bit_access: vk::TRUE,
      ..Default::default()
    };

    let mut vk_physical_device_buffer_device_address_features =
      vk::PhysicalDeviceBufferDeviceAddressFeatures {
        buffer_device_address: vk::TRUE,
        p_next: (&raw mut vk_physical_device_8_bit_storage_features).cast(),
        ..Default::default()
      };

    let mut vk_physical_device_vulkan_memory_model_features =
      vk::PhysicalDeviceVulkanMemoryModelFeatures {
        vulkan_memory_model: vk::TRUE,
        vulkan_memory_model_device_scope: vk::TRUE,
        p_next: (&raw mut vk_physical_device_buffer_device_address_features).cast(),
        ..Default::default()
      };

    let mut vk_physical_device_timeline_semaphore_features =
      vk::PhysicalDeviceTimelineSemaphoreFeatures {
        timeline_semaphore: vk::TRUE,
        p_next: (&raw mut vk_physical_device_vulkan_memory_model_features).cast(),
        ..Default::default()
      };

    let mut vk_physical_device_pageable_device_local_memory_features =
      vk::PhysicalDevicePageableDeviceLocalMemoryFeaturesEXT {
        pageable_device_local_memory: vk::TRUE,
        p_next: (&raw mut vk_physical_device_timeline_semaphore_features).cast(),
        ..Default::default()
      };

    let vk_physical_device_features = vk::PhysicalDeviceFeatures2 {
      features: vk::PhysicalDeviceFeatures {
        fragment_stores_and_atomics: vk::TRUE,
        vertex_pipeline_stores_and_atomics: vk::TRUE,
        shader_int64: vk::TRUE,
        ..Default::default()
      },
      p_next: (&raw mut vk_physical_device_pageable_device_local_memory_features).cast(),
      ..Default::default()
    };

    let vk_device_create_info = vk::DeviceCreateInfo {
      queue_create_info_count: vk_queue_create_info_count,
      p_queue_create_infos: vk_queue_create_infos.as_ptr(),
      enabled_extension_count: vk_device_ext_count,
      pp_enabled_extension_names: vk_device_ext_names.as_ptr(),
      p_next: (&raw const vk_physical_device_features).cast(),
      ..Default::default()
    };

    let vk_device = match unsafe {
      vk_instance.create_device(vk_physical_device, &vk_device_create_info, None)
    } {
      Ok(device) => device,
      Err(err) => panic!("{err}"),
    };

    let graphics_queue_info = vk::DeviceQueueInfo2 {
      queue_family_index: graphics_queue_family_index,
      queue_index: 0,
      ..Default::default()
    };

    let graphics_queue = unsafe { vk_device.get_device_queue2(&graphics_queue_info) };

    window.set_visible(true);

    Self {
      window,
      vk_entry,
      vk_instance,
      vk_device,
    }
  }
}

impl Drop for App {
  fn drop(&mut self) {
    unsafe {
      self.vk_device.destroy_device(None);
    }
    unsafe {
      self.vk_instance.destroy_instance(None);
    }
  }
}
