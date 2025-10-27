use ash::vk::{self, Handle};
use optarg2chain::optarg_fn;
use rustc_hash::FxHashSet;
use sdl3::{event::Event, image::LoadSurface, surface::Surface};
use std::{borrow::Cow, ffi::CString, iter};

#[optarg_fn(RunBuilder, call)]
pub fn run(
  #[optarg_default] title: Cow<'static, str>,
  #[optarg((800, 600))] size: (u32, u32),
  #[optarg_default] favicon_path: Cow<'static, str>,
) {
  let sdl = sdl3::init().unwrap();
  sdl3::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");
  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window(&title, size.0, size.1)
    .high_pixel_density()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  if let Ok(favicon) = Surface::from_file(&*favicon_path) {
    window.set_icon(favicon);
  }

  let ash_entry = unsafe { ash::Entry::load().unwrap() };
  let app_name = CString::new(&*title).unwrap();

  let vk_app_info = vk::ApplicationInfo {
    p_application_name: app_name.as_ptr(),
    api_version: vk::API_VERSION_1_3,
    ..Default::default()
  };

  let vk_layer_names = [
    #[cfg(debug_assertions)]
    CString::new("VK_LAYER_KHRONOS_validation").unwrap(),
  ];

  let vk_layer_names = vk_layer_names
    .iter()
    .map(|layer_name: &CString| layer_name.as_ptr())
    .collect::<Box<_>>();

  let vk_ext_names = window.vulkan_instance_extensions().unwrap();

  let vk_ext_names = vk_ext_names
    .iter()
    .map(|ext_name| ext_name.as_ptr() as *const _)
    .chain(iter::once(vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr()))
    .collect::<Box<_>>();

  #[cfg(debug_assertions)]
  let vk_inst = {
    let validation_layer_name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();

    let gpuav_enable_setting_name = CString::new("gpuav_enable").unwrap();
    let gpuav_enable_true = vk::TRUE;

    let gpuav_validate_ray_query_setting_name = CString::new("gpuav_validate_ray_query").unwrap();
    let gpuav_validate_ray_query_false = vk::FALSE;

    let report_flags_setting_name = CString::new("report_flags").unwrap();
    let report_flag_error = CString::new("error").unwrap();
    let report_flag_perf = CString::new("perf").unwrap();
    let report_flag_warn = CString::new("warn").unwrap();
    let report_flags = [
      report_flag_error.as_ptr(),
      report_flag_perf.as_ptr(),
      report_flag_warn.as_ptr(),
    ];

    let validate_best_practices_setting_name = CString::new("validate_best_practices").unwrap();
    let validate_best_practices_true = vk::TRUE;

    let validate_best_practices_amd_setting_name =
      CString::new("validate_best_practices_amd").unwrap();
    let validate_best_practices_amd_true = vk::TRUE;

    let validate_best_practices_arm_setting_name =
      CString::new("validate_best_practices_arm").unwrap();
    let validate_best_practices_arm_true = vk::TRUE;

    let validate_best_practices_img_setting_name =
      CString::new("validate_best_practices_img").unwrap();
    let validate_best_practices_img_true = vk::TRUE;

    let validate_best_practices_nvidia_setting_name =
      CString::new("validate_best_practices_nvidia").unwrap();
    let validate_best_practices_nvidia_true = vk::TRUE;

    let validate_sync_setting_name = CString::new("validate_sync").unwrap();
    let validate_sync_true = vk::TRUE;

    let vk_layer_settings = [
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: gpuav_enable_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: &gpuav_enable_true as *const _ as *const _,
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: gpuav_validate_ray_query_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: &gpuav_validate_ray_query_false as *const _ as *const _,
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: report_flags_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::STRING,
        value_count: report_flags.len() as _,
        p_values: report_flags.as_ptr() as *const _,
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: &validate_best_practices_true as *const _ as *const _,
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_amd_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: &validate_best_practices_amd_true as *const _ as *const _,
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_arm_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: &validate_best_practices_arm_true as *const _ as *const _,
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_img_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: &validate_best_practices_img_true as *const _ as *const _,
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: validate_best_practices_nvidia_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: &validate_best_practices_nvidia_true as *const _ as *const _,
        ..Default::default()
      },
      vk::LayerSettingEXT {
        p_layer_name: validation_layer_name.as_ptr(),
        p_setting_name: validate_sync_setting_name.as_ptr(),
        ty: vk::LayerSettingTypeEXT::BOOL32,
        value_count: 1,
        p_values: &validate_sync_true as *const _ as *const _,
        ..Default::default()
      },
    ];

    let vk_layer_settings_create_info = vk::LayerSettingsCreateInfoEXT {
      setting_count: vk_layer_settings.len() as _,
      p_settings: vk_layer_settings.as_ptr(),
      ..Default::default()
    };

    let vk_inst_create_info = vk::InstanceCreateInfo {
      flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
      p_application_info: &vk_app_info,
      enabled_layer_count: vk_layer_names.len() as _,
      pp_enabled_layer_names: vk_layer_names.as_ptr(),
      enabled_extension_count: vk_ext_names.len() as _,
      pp_enabled_extension_names: vk_ext_names.as_ptr(),
      p_next: &vk_layer_settings_create_info as *const _ as *const _,
      ..Default::default()
    };

    unsafe {
      ash_entry
        .create_instance(&vk_inst_create_info, None)
        .unwrap()
    }
  };

  #[cfg(not(debug_assertions))]
  let vk_inst = {
    let vk_inst_create_info = vk::InstanceCreateInfo {
      flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
      p_application_info: &vk_app_info,
      enabled_layer_count: vk_layer_names.len() as _,
      pp_enabled_layer_names: vk_layer_names.as_ptr(),
      enabled_extension_count: vk_ext_names.len() as _,
      pp_enabled_extension_names: vk_ext_names.as_ptr(),
      ..Default::default()
    };

    unsafe {
      ash_entry
        .create_instance(&vk_inst_create_info, None)
        .unwrap()
    }
  };

  let vk_surface = vk::SurfaceKHR::from_raw(
    window
      .vulkan_create_surface(vk_inst.handle().as_raw() as *mut _)
      .unwrap() as _,
  );

  let surface_inst = ash::khr::surface::Instance::new(&ash_entry, &vk_inst);
  let vk_physical_devices = unsafe { vk_inst.enumerate_physical_devices().unwrap() };

  let (vk_physical_device, graphics_queue_family_index, present_queue_family_index) =
    vk_physical_devices
      .into_iter()
      .filter_map(|vk_physical_device| {
        let queue_family_props_count =
          unsafe { vk_inst.get_physical_device_queue_family_properties2_len(vk_physical_device) };

        let mut queue_family_props =
          vec![vk::QueueFamilyProperties2::default(); queue_family_props_count];

        unsafe {
          vk_inst.get_physical_device_queue_family_properties2(
            vk_physical_device,
            &mut queue_family_props,
          );
        }

        let graphics_queue_family_index = queue_family_props.iter().enumerate().find_map(
          |(queue_family_index, queue_family_props)| {
            if queue_family_props
              .queue_family_properties
              .queue_flags
              .contains(vk::QueueFlags::GRAPHICS)
            {
              Some(queue_family_index)
            } else {
              None
            }
          },
        );

        let present_queue_family_index =
          queue_family_props
            .iter()
            .enumerate()
            .find_map(|(queue_family_index, _)| unsafe {
              if surface_inst
                .get_physical_device_surface_support(
                  vk_physical_device,
                  queue_family_index as _,
                  vk_surface,
                )
                .unwrap()
              {
                Some(queue_family_index)
              } else {
                None
              }
            });

        if let (Some(graphics_queue_family_index), Some(present_queue_family_index)) =
          (graphics_queue_family_index, present_queue_family_index)
        {
          Some((
            vk_physical_device,
            graphics_queue_family_index,
            present_queue_family_index,
          ))
        } else {
          None
        }
      })
      .max_by_key(|&(vk_physical_device, _, _)| {
        let mut props = vk::PhysicalDeviceProperties2::default();
        unsafe { vk_inst.get_physical_device_properties2(vk_physical_device, &mut props) };

        match props.properties.device_type {
          vk::PhysicalDeviceType::INTEGRATED_GPU => 4,
          vk::PhysicalDeviceType::DISCRETE_GPU => 3,
          vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
          vk::PhysicalDeviceType::CPU => 1,
          _ => 0,
        }
      })
      .unwrap();

  let queue_priorities = [1.0];

  let queue_create_infos =
    FxHashSet::from_iter([graphics_queue_family_index, present_queue_family_index])
      .into_iter()
      .map(|queue_family_index| vk::DeviceQueueCreateInfo {
        queue_family_index: queue_family_index as _,
        queue_count: queue_priorities.len() as _,
        p_queue_priorities: queue_priorities.as_ptr(),
        ..Default::default()
      })
      .collect::<Box<_>>();

  let vk_ext_props = unsafe {
    vk_inst
      .enumerate_device_extension_properties(vk_physical_device)
      .unwrap()
  };

  let has_portability_subset = vk_ext_props.into_iter().any(|vk_ext_props| {
    let vk_ext_name = vk_ext_props.extension_name_as_c_str().unwrap();
    vk_ext_name == vk::KHR_PORTABILITY_SUBSET_NAME
  });

  let mut vk_device_ext_names = vec![
    vk::KHR_SWAPCHAIN_NAME.as_ptr(),
    vk::EXT_PAGEABLE_DEVICE_LOCAL_MEMORY_NAME.as_ptr(),
    vk::EXT_MEMORY_PRIORITY_NAME.as_ptr(),
  ];

  if has_portability_subset {
    vk_device_ext_names.push(vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr());
  }

  let vk_physical_device_pageable_device_local_memory_features =
    vk::PhysicalDevicePageableDeviceLocalMemoryFeaturesEXT {
      pageable_device_local_memory: vk::TRUE,
      ..Default::default()
    };

  let vk_physical_device_vulkan_12_features = vk::PhysicalDeviceVulkan12Features {
    buffer_device_address: vk::TRUE,
    timeline_semaphore: vk::TRUE,
    vulkan_memory_model: vk::TRUE,
    vulkan_memory_model_device_scope: vk::TRUE,
    uniform_and_storage_buffer8_bit_access: vk::TRUE,
    p_next: &vk_physical_device_pageable_device_local_memory_features as *const _ as *mut _,
    ..Default::default()
  };

  let vk_physical_device_features = vk::PhysicalDeviceFeatures2 {
    features: vk::PhysicalDeviceFeatures {
      vertex_pipeline_stores_and_atomics: vk::TRUE,
      fragment_stores_and_atomics: vk::TRUE,
      shader_int64: vk::TRUE,
      ..Default::default()
    },
    p_next: &vk_physical_device_vulkan_12_features as *const _ as *mut _,
    ..Default::default()
  };

  let vk_device_create_info = vk::DeviceCreateInfo {
    queue_create_info_count: queue_create_infos.len() as _,
    p_queue_create_infos: queue_create_infos.as_ptr(),
    enabled_extension_count: vk_device_ext_names.len() as _,
    pp_enabled_extension_names: vk_device_ext_names.as_ptr(),
    p_next: &vk_physical_device_features as *const _ as *const _,
    ..Default::default()
  };

  let vk_device = unsafe {
    vk_inst
      .create_device(vk_physical_device, &vk_device_create_info, None)
      .unwrap()
  };

  let graphics_queue_info = vk::DeviceQueueInfo2 {
    queue_family_index: graphics_queue_family_index as _,
    queue_index: 0,
    ..Default::default()
  };

  let present_queue_info = vk::DeviceQueueInfo2 {
    queue_family_index: present_queue_family_index as _,
    queue_index: 0,
    ..Default::default()
  };

  let graphics_queue = unsafe { vk_device.get_device_queue2(&graphics_queue_info) };
  let present_queue = unsafe { vk_device.get_device_queue2(&present_queue_info) };

  let mut event_pump = sdl.event_pump().unwrap();

  'running: loop {
    for event in event_pump.poll_iter() {
      if let Event::Quit { .. } = event {
        break 'running;
      }
    }
  }

  unsafe {
    vk_device.destroy_device(None);
    surface_inst.destroy_surface(vk_surface, None);
    vk_inst.destroy_instance(None);
  }
}
