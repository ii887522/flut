use crate::{
  consts,
  model_sync::ModelSync,
  models::{
    Model as _, model_capacities::ModelCapacities, push_consts::PushConsts, round_rect::RoundRect,
  },
  storage_buffer::StorageBuffer,
};
use ash::{khr, vk};
use rustc_hash::FxHashSet;
use std::{
  ffi::{CStr, CString, c_char},
  iter, mem, ptr, slice,
};
use vk_mem::Alloc as _;
use winit::{
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
  event_loop::ActiveEventLoop,
  raw_window_handle::{HasDisplayHandle as _, HasWindowHandle as _},
  window::{Window, WindowButtons},
};

const VK_INSTANCE_EXT_NAMES: &[*const c_char] = &[
  #[cfg(debug_assertions)]
  vk::EXT_LAYER_SETTINGS_NAME.as_ptr(),
  vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr(),
];

const VK_DEVICE_EXT_NAMES: &[&CStr] = &[
  vk::EXT_MEMORY_BUDGET_NAME,
  vk::EXT_MEMORY_PRIORITY_NAME,
  vk::EXT_PAGEABLE_DEVICE_LOCAL_MEMORY_NAME,
  vk::KHR_PORTABILITY_SUBSET_NAME,
  vk::KHR_SWAPCHAIN_NAME,
];

const VERT_SHADER_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shader.vert.spv"));
const FRAG_SHADER_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shader.frag.spv"));
const DYNAMIC_STATES: &[vk::DynamicState] =
  &[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

struct WindowMinimized;

struct Shared {
  window: Window,
  _vk_entry: ash::Entry,
  vk_instance: ash::Instance,
  vk_surface_instance: khr::surface::Instance,
  vk_surface: vk::SurfaceKHR,
  vk_physical_device: vk::PhysicalDevice,
  graphics_queue_family_index: u32,
  present_queue_family_index: u32,
  vk_device: ash::Device,
  graphics_queue: vk::Queue,
  present_queue: vk::Queue,
  transfer_queue: vk::Queue,
  swapchain_format: vk::SurfaceFormatKHR,
  vk_swapchain_device: khr::swapchain::Device,
  pipeline_layout: vk::PipelineLayout,
  render_pass: vk::RenderPass,
  graphics_pipeline: vk::Pipeline,
  vk_allocator: vk_mem::Allocator,
  model_buffer: StorageBuffer,
  round_rect_sync: ModelSync<RoundRect>,
  msaa_sample_count: vk::SampleCountFlags,
  graphics_command_pools: Box<[vk::CommandPool]>,
  graphics_command_buffers: Box<[vk::CommandBuffer]>,
  image_avail_semaphores: Box<[vk::Semaphore]>,
  render_done_semaphores: Box<[vk::Semaphore]>,
  transfer_done_semaphores: Box<[vk::Semaphore]>,
  in_flight_fences: Box<[vk::Fence]>,
  frame_index: usize,
}

impl Shared {
  fn new(
    event_loop: &ActiveEventLoop,
    title: &str,
    size: (f64, f64),
    model_capacities: ModelCapacities,
  ) -> Self {
    let (width, height) = size;
    let ModelCapacities {
      round_rect_capacity,
    } = model_capacities;

    let window = event_loop
      .create_window(
        Window::default_attributes()
          .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
          .with_inner_size(Size::Logical(LogicalSize::new(width, height)))
          .with_position(Position::Logical(LogicalPosition::new(8_f64, 8_f64)))
          .with_resizable(false)
          .with_title(title)
          .with_visible(false),
      )
      .unwrap();

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

    let vk_entry = unsafe { ash::Entry::load().unwrap() };

    let vk_app_info = vk::ApplicationInfo {
      api_version: vk::API_VERSION_1_3,
      ..Default::default()
    };

    #[cfg(debug_assertions)]
    let vk_validation_layer_name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();

    #[cfg(debug_assertions)]
    let validate_sync_name = CString::new("validate_sync").unwrap();

    #[cfg(debug_assertions)]
    let syncval_shader_accesses_heuristic_name =
      CString::new("syncval_shader_accesses_heuristic").unwrap();

    #[cfg(debug_assertions)]
    let printf_enable_name = CString::new("printf_enable").unwrap();

    #[cfg(debug_assertions)]
    let gpuav_enable_name = CString::new("gpuav_enable").unwrap();

    #[cfg(debug_assertions)]
    let gpuav_validate_ray_query_name = CString::new("gpuav_validate_ray_query").unwrap();

    #[cfg(debug_assertions)]
    let validate_best_practices_name = CString::new("validate_best_practices").unwrap();

    #[cfg(debug_assertions)]
    let validate_best_practices_arm_name = CString::new("validate_best_practices_arm").unwrap();

    #[cfg(debug_assertions)]
    let validate_best_practices_amd_name = CString::new("validate_best_practices_amd").unwrap();

    #[cfg(debug_assertions)]
    let validate_best_practices_img_name = CString::new("validate_best_practices_img").unwrap();

    #[cfg(debug_assertions)]
    let validate_best_practices_nvidia_name =
      CString::new("validate_best_practices_nvidia").unwrap();

    #[cfg(debug_assertions)]
    let report_flags_name = CString::new("report_flags").unwrap();

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
      CString::new("info").unwrap(),
      CString::new("warn").unwrap(),
      CString::new("perf").unwrap(),
      CString::new("error").unwrap(),
    ];

    #[cfg(debug_assertions)]
    let report_flags_values = report_flags_values
      .iter()
      .map(|value| value.as_ptr())
      .collect::<Box<_>>();

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
        value_count: report_flags_values.len().try_into().unwrap(),
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

    let window_handle = window.window_handle().unwrap();
    let display_handle = window.display_handle().unwrap();

    let vk_instance_ext_names =
      ash_window::enumerate_required_extensions(display_handle.as_raw()).unwrap();

    let vk_instance_ext_names = vk_instance_ext_names
      .iter()
      .chain(VK_INSTANCE_EXT_NAMES)
      .copied()
      .collect::<Box<_>>();

    #[cfg(debug_assertions)]
    let vk_layer_settings_create_info = vk::LayerSettingsCreateInfoEXT {
      setting_count: vk_layer_settings.len().try_into().unwrap(),
      p_settings: vk_layer_settings.as_ptr(),
      ..Default::default()
    };

    let vk_instance_create_info = vk::InstanceCreateInfo {
      flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
      p_application_info: &raw const vk_app_info,
      enabled_layer_count: vk_layer_names.len().try_into().unwrap(),
      pp_enabled_layer_names: vk_layer_names.as_ptr(),
      enabled_extension_count: vk_instance_ext_names.len().try_into().unwrap(),
      pp_enabled_extension_names: vk_instance_ext_names.as_ptr(),

      #[cfg(debug_assertions)]
      p_next: (&raw const vk_layer_settings_create_info).cast(),

      ..Default::default()
    };

    let vk_instance = unsafe {
      vk_entry
        .create_instance(&vk_instance_create_info, None)
        .unwrap()
    };

    let vk_surface_instance = khr::surface::Instance::new(&vk_entry, &vk_instance);

    let vk_surface = unsafe {
      ash_window::create_surface(
        &vk_entry,
        &vk_instance,
        display_handle.as_raw(),
        window_handle.as_raw(),
        None,
      )
      .unwrap()
    };

    let vk_physical_devices = unsafe { vk_instance.enumerate_physical_devices().unwrap() };

    let Some((
      vk_physical_device,
      vk_physical_device_props,
      graphics_queue_family_index,
      present_queue_family_index,
      transfer_queue_family_index,
    )) =
      vk_physical_devices
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

          let graphics_queue_family_index = queue_family_props
            .iter()
            .enumerate()
            .filter(|&(_index, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .contains(vk::QueueFlags::GRAPHICS)
            })
            .min_by_key(|&(_index, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .as_raw()
                .count_ones()
            })
            .map(|(index, _queue_family_props)| index.try_into().unwrap())?;

          let present_queue_family_index = queue_family_props.iter().enumerate().find_map(
            |(index, _queue_family_props)| {
              let index = index.try_into().unwrap();

              unsafe {
                vk_surface_instance
                  .get_physical_device_surface_support(vk_physical_device, index, vk_surface)
                  .unwrap_or_else(|err| {
                    eprintln!("Failed to check physical device supports the surface. Assume not supported: {err}");
                    false
                  })
                  .then_some(index)
              }
            }
          )?;

          let transfer_queue_family_index = queue_family_props
            .iter()
            .enumerate()
            .filter(|&(_index, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .contains(vk::QueueFlags::TRANSFER)
            })
            .min_by_key(|&(_index, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .as_raw()
                .count_ones()
            })
            .map_or(
              graphics_queue_family_index,
              |(index, _queue_family_props)| index.try_into().unwrap()
            );

          let mut vk_physical_device_props = vk::PhysicalDeviceProperties2::default();

          unsafe {
            vk_instance
              .get_physical_device_properties2(vk_physical_device, &mut vk_physical_device_props);
          }

          Some((
            vk_physical_device,
            vk_physical_device_props,
            graphics_queue_family_index,
            present_queue_family_index,
            transfer_queue_family_index,
          ))
        })
        .max_by_key(
          |&(
            _vk_physical_device,
            vk_physical_device_props,
            _graphics_queue_family_index,
            _present_queue_family_index,
            _transfer_queue_family_index
          )| {
            match vk_physical_device_props.properties.device_type {
              vk::PhysicalDeviceType::INTEGRATED_GPU => 4_u32,
              vk::PhysicalDeviceType::DISCRETE_GPU => 3_u32,
              vk::PhysicalDeviceType::VIRTUAL_GPU => 2_u32,
              vk::PhysicalDeviceType::CPU => 1_u32,
              _ => 0_u32,
            }
          },
        )
    else {
      panic!("No suitable physical device found");
    };

    let queue_priorities = [1.0];

    let vk_queue_create_infos = FxHashSet::from_iter([
      graphics_queue_family_index,
      present_queue_family_index,
      transfer_queue_family_index,
    ])
    .into_iter()
    .map(|queue_family_index| vk::DeviceQueueCreateInfo {
      queue_family_index,
      queue_count: queue_priorities.len().try_into().unwrap(),
      p_queue_priorities: queue_priorities.as_ptr(),
      ..Default::default()
    })
    .collect::<Box<_>>();

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

        VK_DEVICE_EXT_NAMES
          .iter()
          .find_map(|&req_vk_ext_name| (req_vk_ext_name == vk_ext_name).then_some(req_vk_ext_name))
      })
      .collect::<Box<_>>();

    let allocator_create_flags = vk_mem::AllocatorCreateFlags::EXTERNALLY_SYNCHRONIZED
      | vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS
      | vk_mem::AllocatorCreateFlags::KHR_DEDICATED_ALLOCATION
      | vk_device_ext_names
        .iter()
        .fold(vk_mem::AllocatorCreateFlags::empty(), |acc, &ext_name| {
          let allocator_create_flag = if ext_name == vk::EXT_MEMORY_BUDGET_NAME {
            vk_mem::AllocatorCreateFlags::EXT_MEMORY_BUDGET
          } else if ext_name == vk::EXT_MEMORY_PRIORITY_NAME {
            vk_mem::AllocatorCreateFlags::EXT_MEMORY_PRIORITY
          } else {
            vk_mem::AllocatorCreateFlags::empty()
          };

          acc | allocator_create_flag
        });

    let vk_device_ext_names = vk_device_ext_names
      .into_iter()
      .map(CStr::as_ptr)
      .collect::<Box<_>>();

    let mut vk_separate_depth_stencil_layouts_features =
      vk::PhysicalDeviceSeparateDepthStencilLayoutsFeatures {
        separate_depth_stencil_layouts: vk::TRUE,
        ..Default::default()
      };

    let mut vk_physical_device_8_bit_storage_features = vk::PhysicalDevice8BitStorageFeatures {
      uniform_and_storage_buffer8_bit_access: vk::TRUE,
      p_next: (&raw mut vk_separate_depth_stencil_layouts_features).cast(),
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
        alpha_to_one: vk::TRUE,
        ..Default::default()
      },
      p_next: (&raw mut vk_physical_device_pageable_device_local_memory_features).cast(),
      ..Default::default()
    };

    let vk_device_create_info = vk::DeviceCreateInfo {
      queue_create_info_count: vk_queue_create_infos.len().try_into().unwrap(),
      p_queue_create_infos: vk_queue_create_infos.as_ptr(),
      enabled_extension_count: vk_device_ext_names.len().try_into().unwrap(),
      pp_enabled_extension_names: vk_device_ext_names.as_ptr(),
      p_next: (&raw const vk_physical_device_features).cast(),
      ..Default::default()
    };

    let vk_device = unsafe {
      vk_instance
        .create_device(vk_physical_device, &vk_device_create_info, None)
        .unwrap()
    };

    let graphics_queue_info = vk::DeviceQueueInfo2 {
      queue_family_index: graphics_queue_family_index,
      queue_index: 0,
      ..Default::default()
    };

    let present_queue_info = vk::DeviceQueueInfo2 {
      queue_family_index: present_queue_family_index,
      queue_index: 0,
      ..Default::default()
    };

    let transfer_queue_info = vk::DeviceQueueInfo2 {
      queue_family_index: transfer_queue_family_index,
      queue_index: 0,
      ..Default::default()
    };

    let graphics_queue = unsafe { vk_device.get_device_queue2(&graphics_queue_info) };
    let present_queue = unsafe { vk_device.get_device_queue2(&present_queue_info) };
    let transfer_queue = unsafe { vk_device.get_device_queue2(&transfer_queue_info) };

    let vk_surface_formats = unsafe {
      vk_surface_instance
        .get_physical_device_surface_formats(vk_physical_device, vk_surface)
        .unwrap()
    };

    let &swapchain_format = vk_surface_formats
      .iter()
      .find(|&vk_surface_format| vk_surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
      .unwrap_or_else(|| &vk_surface_formats[0]);

    let vk_swapchain_device = khr::swapchain::Device::new(&vk_instance, &vk_device);

    let max_msaa_sample_count = vk_physical_device_props
      .properties
      .limits
      .framebuffer_color_sample_counts
      & vk_physical_device_props
        .properties
        .limits
        .framebuffer_depth_sample_counts;

    let msaa_sample_count = if max_msaa_sample_count.contains(vk::SampleCountFlags::TYPE_4) {
      vk::SampleCountFlags::TYPE_4
    } else if max_msaa_sample_count.contains(vk::SampleCountFlags::TYPE_2) {
      vk::SampleCountFlags::TYPE_2
    } else {
      vk::SampleCountFlags::TYPE_1
    };

    let vert_shader_module_create_info = vk::ShaderModuleCreateInfo {
      code_size: VERT_SHADER_CODE.len(),
      p_code: VERT_SHADER_CODE.as_ptr().cast(),
      ..Default::default()
    };

    let frag_shader_module_create_info = vk::ShaderModuleCreateInfo {
      code_size: FRAG_SHADER_CODE.len(),
      p_code: FRAG_SHADER_CODE.as_ptr().cast(),
      ..Default::default()
    };

    let vert_shader_module = unsafe {
      vk_device
        .create_shader_module(&vert_shader_module_create_info, None)
        .unwrap()
    };

    let frag_shader_module = unsafe {
      vk_device
        .create_shader_module(&frag_shader_module_create_info, None)
        .unwrap()
    };

    let main_name = CString::new("main").unwrap();

    let shader_stage_create_infos = [
      vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::VERTEX,
        module: vert_shader_module,
        p_name: main_name.as_ptr(),
        ..Default::default()
      },
      vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: frag_shader_module,
        p_name: main_name.as_ptr(),
        ..Default::default()
      },
    ];

    let vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::default();

    let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo {
      topology: vk::PrimitiveTopology::TRIANGLE_LIST,
      ..Default::default()
    };

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
      viewport_count: 1,
      scissor_count: 1,
      ..Default::default()
    };

    let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo {
      front_face: vk::FrontFace::CLOCKWISE,
      line_width: 1.0,
      ..Default::default()
    };

    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
      rasterization_samples: msaa_sample_count,
      alpha_to_coverage_enable: vk::TRUE,
      alpha_to_one_enable: vk::TRUE,
      ..Default::default()
    };

    let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
      depth_test_enable: vk::TRUE,
      depth_write_enable: vk::TRUE,
      depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
      ..Default::default()
    };

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
      color_write_mask: vk::ColorComponentFlags::RGBA,
      ..Default::default()
    }];

    let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo {
      attachment_count: color_blend_attachment_states.len().try_into().unwrap(),
      p_attachments: color_blend_attachment_states.as_ptr(),
      ..Default::default()
    };

    let dynamic_state_create_info = vk::PipelineDynamicStateCreateInfo {
      dynamic_state_count: DYNAMIC_STATES.len().try_into().unwrap(),
      p_dynamic_states: DYNAMIC_STATES.as_ptr(),
      ..Default::default()
    };

    let push_const_ranges = [vk::PushConstantRange {
      stage_flags: vk::ShaderStageFlags::VERTEX,
      offset: 0,
      size: mem::size_of::<PushConsts>().try_into().unwrap(),
    }];

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
      push_constant_range_count: push_const_ranges.len().try_into().unwrap(),
      p_push_constant_ranges: push_const_ranges.as_ptr(),
      ..Default::default()
    };

    let pipeline_layout = unsafe {
      vk_device
        .create_pipeline_layout(&pipeline_layout_create_info, None)
        .unwrap()
    };

    // Attachment 0: MSAA color attachment (or direct if 1x)
    // Attachment 1: Depth attachment
    // Attachment 2: Resolve attachment (swapchain image) - only used when MSAA > 1x
    let mut attachment_descs = vec![if msaa_sample_count == vk::SampleCountFlags::TYPE_1 {
      // Color attachment
      vk::AttachmentDescription2 {
        format: swapchain_format.format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
      }
    } else {
      // MSAA color attachment
      vk::AttachmentDescription2 {
        format: swapchain_format.format,
        samples: msaa_sample_count,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
      }
    }];

    // Depth attachment
    attachment_descs.push(vk::AttachmentDescription2 {
      format: vk::Format::D16_UNORM,
      samples: msaa_sample_count,
      load_op: vk::AttachmentLoadOp::CLEAR,
      store_op: vk::AttachmentStoreOp::DONT_CARE,
      stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
      stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
      initial_layout: vk::ImageLayout::UNDEFINED,
      final_layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
      ..Default::default()
    });

    if msaa_sample_count != vk::SampleCountFlags::TYPE_1 {
      // Resolve attachment (swapchain image)
      attachment_descs.push(vk::AttachmentDescription2 {
        format: swapchain_format.format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::DONT_CARE,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
      });
    }

    let attachment_descs = attachment_descs;

    let color_attachment_refs = [vk::AttachmentReference2 {
      attachment: 0,
      layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
      ..Default::default()
    }];

    let depth_attachment_ref = vk::AttachmentReference2 {
      attachment: 1,
      layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
      ..Default::default()
    };

    let resolve_attachment_refs = [vk::AttachmentReference2 {
      attachment: 2,
      layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
      ..Default::default()
    }];

    let subpass_descs = [vk::SubpassDescription2 {
      pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
      color_attachment_count: color_attachment_refs.len().try_into().unwrap(),
      p_color_attachments: color_attachment_refs.as_ptr(),
      p_depth_stencil_attachment: &raw const depth_attachment_ref,
      p_resolve_attachments: if msaa_sample_count == vk::SampleCountFlags::TYPE_1 {
        ptr::null()
      } else {
        resolve_attachment_refs.as_ptr()
      },
      ..Default::default()
    }];

    let subpass_deps = [vk::SubpassDependency2 {
      src_subpass: vk::SUBPASS_EXTERNAL,
      dst_subpass: 0,
      src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
      dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
      src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
      dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
      dependency_flags: vk::DependencyFlags::BY_REGION,
      ..Default::default()
    }];

    let render_pass_create_info = vk::RenderPassCreateInfo2 {
      attachment_count: attachment_descs.len().try_into().unwrap(),
      p_attachments: attachment_descs.as_ptr(),
      subpass_count: subpass_descs.len().try_into().unwrap(),
      p_subpasses: subpass_descs.as_ptr(),
      dependency_count: subpass_deps.len().try_into().unwrap(),
      p_dependencies: subpass_deps.as_ptr(),
      ..Default::default()
    };

    let render_pass = unsafe {
      vk_device
        .create_render_pass2(&render_pass_create_info, None)
        .unwrap()
    };

    let graphics_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo {
      stage_count: shader_stage_create_infos.len().try_into().unwrap(),
      p_stages: shader_stage_create_infos.as_ptr(),
      p_vertex_input_state: &raw const vert_input_state_create_info,
      p_input_assembly_state: &raw const input_assembly_state_create_info,
      p_viewport_state: &raw const viewport_state_create_info,
      p_rasterization_state: &raw const rasterization_state_create_info,
      p_multisample_state: &raw const multisample_state_create_info,
      p_depth_stencil_state: &raw const depth_stencil_state_create_info,
      p_color_blend_state: &raw const color_blend_state_create_info,
      p_dynamic_state: &raw const dynamic_state_create_info,
      layout: pipeline_layout,
      render_pass,
      subpass: 0,
      base_pipeline_index: -1,
      ..Default::default()
    }];

    let graphics_pipeline = unsafe {
      vk_device
        .create_graphics_pipelines(
          vk::PipelineCache::null(),
          &graphics_pipeline_create_infos,
          None,
        )
        .unwrap()[0]
    };

    unsafe {
      vk_device.destroy_shader_module(frag_shader_module, None);
    }
    unsafe {
      vk_device.destroy_shader_module(vert_shader_module, None);
    }

    let total_model_capacity_bytes = model_capacities.calc_bytes();

    let mut vk_allocator_create_info =
      vk_mem::AllocatorCreateInfo::new(&vk_instance, &vk_device, vk_physical_device);

    vk_allocator_create_info.flags = allocator_create_flags;
    vk_allocator_create_info.preferred_large_heap_block_size =
      (consts::MAX_IN_FLIGHT_FRAME_COUNT * total_model_capacity_bytes).max(2 * 1024 * 1024) as u64; // Min 2 MB
    vk_allocator_create_info.vulkan_api_version = vk::API_VERSION_1_3;

    let vk_allocator = unsafe { vk_mem::Allocator::new(vk_allocator_create_info).unwrap() };

    let model_buffer = StorageBuffer::new(
      &vk_device,
      &vk_allocator,
      graphics_queue_family_index,
      transfer_queue_family_index,
      total_model_capacity_bytes,
    );

    let round_rect_sync = ModelSync::new(round_rect_capacity);

    let graphics_command_pools = iter::repeat_with(|| {
      let command_pool_create_info = vk::CommandPoolCreateInfo {
        flags: vk::CommandPoolCreateFlags::TRANSIENT,
        queue_family_index: graphics_queue_family_index,
        ..Default::default()
      };

      unsafe {
        vk_device
          .create_command_pool(&command_pool_create_info, None)
          .unwrap()
      }
    })
    .take(consts::MAX_IN_FLIGHT_FRAME_COUNT)
    .collect::<Box<_>>();

    let graphics_command_buffers = graphics_command_pools
      .iter()
      .map(|&command_pool| {
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
          command_pool,
          level: vk::CommandBufferLevel::PRIMARY,
          command_buffer_count: 1,
          ..Default::default()
        };

        unsafe {
          vk_device
            .allocate_command_buffers(&command_buffer_alloc_info)
            .unwrap()[0]
        }
      })
      .collect::<Box<_>>();

    let semaphore_create_info = vk::SemaphoreCreateInfo::default();

    let fence_create_info = vk::FenceCreateInfo {
      flags: vk::FenceCreateFlags::SIGNALED,
      ..Default::default()
    };

    let image_avail_semaphores = iter::repeat_with(|| unsafe {
      vk_device
        .create_semaphore(&semaphore_create_info, None)
        .unwrap()
    })
    .take(consts::MAX_IN_FLIGHT_FRAME_COUNT)
    .collect::<Box<_>>();

    let render_done_semaphores = iter::repeat_with(|| unsafe {
      vk_device
        .create_semaphore(&semaphore_create_info, None)
        .unwrap()
    })
    .take(consts::MAX_IN_FLIGHT_FRAME_COUNT)
    .collect::<Box<_>>();

    let transfer_done_semaphores = iter::repeat_with(|| unsafe {
      vk_device
        .create_semaphore(&semaphore_create_info, None)
        .unwrap()
    })
    .take(consts::MAX_IN_FLIGHT_FRAME_COUNT)
    .collect::<Box<_>>();

    let in_flight_fences =
      iter::repeat_with(|| unsafe { vk_device.create_fence(&fence_create_info, None).unwrap() })
        .take(consts::MAX_IN_FLIGHT_FRAME_COUNT)
        .collect::<Box<_>>();

    Self {
      window,
      _vk_entry: vk_entry,
      vk_instance,
      vk_surface_instance,
      vk_surface,
      vk_physical_device,
      graphics_queue_family_index,
      present_queue_family_index,
      vk_device,
      graphics_queue,
      present_queue,
      transfer_queue,
      swapchain_format,
      vk_swapchain_device,
      pipeline_layout,
      render_pass,
      graphics_pipeline,
      vk_allocator,
      model_buffer,
      round_rect_sync,
      msaa_sample_count,
      graphics_command_pools,
      graphics_command_buffers,
      image_avail_semaphores,
      render_done_semaphores,
      transfer_done_semaphores,
      in_flight_fences,
      frame_index: 0,
    }
  }

  fn drop(self) {
    unsafe {
      self
        .in_flight_fences
        .iter()
        .for_each(|&fence| self.vk_device.destroy_fence(fence, None));
    }
    unsafe {
      self
        .transfer_done_semaphores
        .iter()
        .for_each(|&semaphore| self.vk_device.destroy_semaphore(semaphore, None));
    }
    unsafe {
      self
        .render_done_semaphores
        .iter()
        .for_each(|&semaphore| self.vk_device.destroy_semaphore(semaphore, None));
    }
    unsafe {
      self
        .image_avail_semaphores
        .iter()
        .for_each(|&semaphore| self.vk_device.destroy_semaphore(semaphore, None));
    }
    unsafe {
      self
        .graphics_command_pools
        .iter()
        .for_each(|&command_pool| self.vk_device.destroy_command_pool(command_pool, None));
    }

    self.model_buffer.drop(&self.vk_device, &self.vk_allocator);
    drop(self.vk_allocator);

    unsafe {
      self
        .vk_device
        .destroy_pipeline(self.graphics_pipeline, None);
    }
    unsafe {
      self.vk_device.destroy_render_pass(self.render_pass, None);
    }
    unsafe {
      self
        .vk_device
        .destroy_pipeline_layout(self.pipeline_layout, None);
    }
    unsafe {
      self.vk_device.destroy_device(None);
    }
    unsafe {
      self
        .vk_surface_instance
        .destroy_surface(self.vk_surface, None);
    }
    unsafe {
      self.vk_instance.destroy_instance(None);
    }
  }
}

pub struct Creating {
  old_swapchain: vk::SwapchainKHR,
}

pub struct Created {
  swapchain: vk::SwapchainKHR,
  _swapchain_images: Box<[vk::Image]>,
  swapchain_image_views: Box<[vk::ImageView]>,
  msaa_image: Option<vk::Image>,
  msaa_image_alloc: Option<vk_mem::Allocation>,
  msaa_image_view: Option<vk::ImageView>,
  depth_image: vk::Image,
  depth_image_alloc: vk_mem::Allocation,
  depth_image_view: vk::ImageView,
  swapchain_extent: vk::Extent2D,
  swapchain_framebuffers: Box<[vk::Framebuffer]>,
}

impl Created {
  fn new(shared: &Shared, old_swapchain: vk::SwapchainKHR) -> Result<Self, WindowMinimized> {
    let vk_surface_caps = unsafe {
      shared
        .vk_surface_instance
        .get_physical_device_surface_capabilities(shared.vk_physical_device, shared.vk_surface)
        .unwrap()
    };

    let swapchain_extent = if vk_surface_caps.current_extent.width < u32::MAX {
      vk_surface_caps.current_extent
    } else {
      let PhysicalSize { width, height } = shared.window.inner_size();

      vk::Extent2D {
        width: width.clamp(
          vk_surface_caps.min_image_extent.width,
          vk_surface_caps.max_image_extent.width,
        ),
        height: height.clamp(
          vk_surface_caps.min_image_extent.height,
          vk_surface_caps.max_image_extent.height,
        ),
      }
    };

    if swapchain_extent.width == 0 || swapchain_extent.height == 0 {
      return Err(WindowMinimized);
    }

    let swapchain_image_count = vk_surface_caps.min_image_count + 1;

    let swapchain_image_count = if vk_surface_caps.max_image_count > 0 {
      swapchain_image_count.min(vk_surface_caps.max_image_count)
    } else {
      swapchain_image_count
    };

    let queue_family_indices = [
      shared.graphics_queue_family_index,
      shared.present_queue_family_index,
    ];

    let (swapchain_image_sharing_mode, swapchain_queue_family_indices) =
      if shared.graphics_queue_family_index == shared.present_queue_family_index {
        (vk::SharingMode::EXCLUSIVE, [].as_slice())
      } else {
        (vk::SharingMode::CONCURRENT, queue_family_indices.as_slice())
      };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
      surface: shared.vk_surface,
      min_image_count: swapchain_image_count,
      image_format: shared.swapchain_format.format,
      image_color_space: shared.swapchain_format.color_space,
      image_extent: swapchain_extent,
      image_array_layers: 1,
      image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
      image_sharing_mode: swapchain_image_sharing_mode,
      queue_family_index_count: swapchain_queue_family_indices.len().try_into().unwrap(),
      p_queue_family_indices: swapchain_queue_family_indices.as_ptr(),
      pre_transform: vk_surface_caps.current_transform,
      composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
      present_mode: vk::PresentModeKHR::FIFO,
      clipped: vk::TRUE,
      old_swapchain,
      ..Default::default()
    };

    let swapchain = unsafe {
      shared
        .vk_swapchain_device
        .create_swapchain(&swapchain_create_info, None)
        .unwrap()
    };

    let swapchain_images = unsafe {
      shared
        .vk_swapchain_device
        .get_swapchain_images(swapchain)
        .unwrap()
    };

    let swapchain_images = swapchain_images.into_boxed_slice();

    let swapchain_image_views = swapchain_images
      .iter()
      .map(|&image| {
        let image_view_create_info = vk::ImageViewCreateInfo {
          image,
          view_type: vk::ImageViewType::TYPE_2D,
          format: shared.swapchain_format.format,
          subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
          },
          ..Default::default()
        };

        unsafe {
          shared
            .vk_device
            .create_image_view(&image_view_create_info, None)
            .unwrap()
        }
      })
      .collect::<Box<_>>();

    // Create MSAA image if using multisampling
    let (msaa_image, msaa_image_alloc, msaa_image_view) =
      if shared.msaa_sample_count == vk::SampleCountFlags::TYPE_1 {
        (None, None, None)
      } else {
        let msaa_image_create_info = vk::ImageCreateInfo {
          image_type: vk::ImageType::TYPE_2D,
          format: shared.swapchain_format.format,
          extent: vk::Extent3D {
            width: swapchain_extent.width,
            height: swapchain_extent.height,
            depth: 1,
          },
          mip_levels: 1,
          array_layers: 1,
          samples: shared.msaa_sample_count,
          tiling: vk::ImageTiling::OPTIMAL,
          usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
          sharing_mode: vk::SharingMode::EXCLUSIVE,
          initial_layout: vk::ImageLayout::UNDEFINED,
          ..Default::default()
        };

        let msaa_image_alloc_create_info = vk_mem::AllocationCreateInfo {
          flags: vk_mem::AllocationCreateFlags::DEDICATED_MEMORY,
          usage: vk_mem::MemoryUsage::AutoPreferDevice,
          preferred_flags: vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
          priority: 1.0,
          ..Default::default()
        };

        let (msaa_image, msaa_image_alloc) = unsafe {
          shared
            .vk_allocator
            .create_image(&msaa_image_create_info, &msaa_image_alloc_create_info)
            .unwrap()
        };

        let msaa_image_view_create_info = vk::ImageViewCreateInfo {
          image: msaa_image,
          view_type: vk::ImageViewType::TYPE_2D,
          format: shared.swapchain_format.format,
          subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
          },
          ..Default::default()
        };

        let msaa_image_view = unsafe {
          shared
            .vk_device
            .create_image_view(&msaa_image_view_create_info, None)
            .unwrap()
        };

        (
          Some(msaa_image),
          Some(msaa_image_alloc),
          Some(msaa_image_view),
        )
      };

    let depth_image_create_info = vk::ImageCreateInfo {
      image_type: vk::ImageType::TYPE_2D,
      format: vk::Format::D16_UNORM,
      extent: vk::Extent3D {
        width: swapchain_extent.width,
        height: swapchain_extent.height,
        depth: 1,
      },
      mip_levels: 1,
      array_layers: 1,
      samples: shared.msaa_sample_count,
      tiling: vk::ImageTiling::OPTIMAL,
      usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
        | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      initial_layout: vk::ImageLayout::UNDEFINED,
      ..Default::default()
    };

    let depth_image_alloc_create_info = vk_mem::AllocationCreateInfo {
      flags: vk_mem::AllocationCreateFlags::DEDICATED_MEMORY,
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      preferred_flags: vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
      priority: 1.0,
      ..Default::default()
    };

    let (depth_image, depth_image_alloc) = unsafe {
      shared
        .vk_allocator
        .create_image(&depth_image_create_info, &depth_image_alloc_create_info)
        .unwrap()
    };

    let depth_image_view_create_info = vk::ImageViewCreateInfo {
      image: depth_image,
      view_type: vk::ImageViewType::TYPE_2D,
      format: vk::Format::D16_UNORM,
      subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::DEPTH,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    let depth_image_view = unsafe {
      shared
        .vk_device
        .create_image_view(&depth_image_view_create_info, None)
        .unwrap()
    };

    let swapchain_framebuffers = swapchain_image_views
      .iter()
      .map(|&swapchain_image_view| {
        let attachments = msaa_image_view.map_or_else(
          || vec![swapchain_image_view, depth_image_view],
          |msaa_view| vec![msaa_view, depth_image_view, swapchain_image_view],
        );

        let framebuffer_create_info = vk::FramebufferCreateInfo {
          render_pass: shared.render_pass,
          attachment_count: attachments.len().try_into().unwrap(),
          p_attachments: attachments.as_ptr(),
          width: swapchain_extent.width,
          height: swapchain_extent.height,
          layers: 1,
          ..Default::default()
        };

        unsafe {
          shared
            .vk_device
            .create_framebuffer(&framebuffer_create_info, None)
            .unwrap()
        }
      })
      .collect::<Box<_>>();

    shared.window.set_visible(true);

    Ok(Self {
      swapchain,
      _swapchain_images: swapchain_images,
      swapchain_image_views,
      msaa_image,
      msaa_image_view,
      msaa_image_alloc,
      depth_image,
      depth_image_view,
      depth_image_alloc,
      swapchain_extent,
      swapchain_framebuffers,
    })
  }

  fn on_swapchain_suboptimal(self, shared: &Shared) -> Result<Self, WindowMinimized> {
    let result = Self::new(shared, self.swapchain);

    unsafe {
      shared.vk_device.device_wait_idle().unwrap();
    }

    self.drop(shared, matches!(result, Err(WindowMinimized)));
    result
  }

  fn drop(mut self, shared: &Shared, skip_swapchain: bool) {
    unsafe {
      self
        .swapchain_framebuffers
        .iter()
        .for_each(|&framebuffer| shared.vk_device.destroy_framebuffer(framebuffer, None));
    }

    unsafe {
      shared
        .vk_device
        .destroy_image_view(self.depth_image_view, None);
    }

    unsafe {
      shared
        .vk_allocator
        .destroy_image(self.depth_image, &mut self.depth_image_alloc);
    }

    if let Some(msaa_image_view) = self.msaa_image_view {
      unsafe {
        shared.vk_device.destroy_image_view(msaa_image_view, None);
      }
    }

    if let (Some(msaa_image), Some(mut msaa_image_alloc)) = (self.msaa_image, self.msaa_image_alloc)
    {
      unsafe {
        shared
          .vk_allocator
          .destroy_image(msaa_image, &mut msaa_image_alloc);
      }
    }

    unsafe {
      self
        .swapchain_image_views
        .iter()
        .for_each(|&image_view| shared.vk_device.destroy_image_view(image_view, None));
    }

    if !skip_swapchain {
      unsafe {
        shared
          .vk_swapchain_device
          .destroy_swapchain(self.swapchain, None);
      }
    }
  }
}

pub struct Renderer<State> {
  shared: Shared,
  state: State,
}

impl Renderer<Creating> {
  pub(super) fn new(
    event_loop: &ActiveEventLoop,
    title: &str,
    size: (f64, f64),
    model_capacities: ModelCapacities,
  ) -> Self {
    Self {
      shared: Shared::new(event_loop, title, size, model_capacities),
      state: Creating {
        old_swapchain: vk::SwapchainKHR::null(),
      },
    }
  }

  pub(super) fn drop(self) {
    let Self { shared, state: _ } = self;

    unsafe {
      shared.vk_device.device_wait_idle().unwrap();
    }

    shared.drop();
  }
}

impl TryFrom<Renderer<Creating>> for Renderer<Created> {
  type Error = Renderer<Creating>;

  fn try_from(renderer: Renderer<Creating>) -> Result<Self, Self::Error> {
    let Renderer { shared, state } = renderer;

    match Created::new(&shared, state.old_swapchain) {
      Ok(created) => {
        unsafe {
          shared
            .vk_swapchain_device
            .destroy_swapchain(state.old_swapchain, None);
        }

        Ok(Self {
          shared,
          state: created,
        })
      }
      Err(WindowMinimized) => Err(Renderer { shared, state }),
    }
  }
}

impl Renderer<Created> {
  pub(super) fn render(self) -> Result<Self, Renderer<Creating>> {
    let Self { mut shared, state } = self;

    let image_avail_semaphore = shared.image_avail_semaphores[shared.frame_index];
    let render_done_semaphore = shared.render_done_semaphores[shared.frame_index];
    let transfer_done_semaphore = shared.transfer_done_semaphores[shared.frame_index];
    let in_flight_fence = shared.in_flight_fences[shared.frame_index];
    let graphics_command_pool = shared.graphics_command_pools[shared.frame_index];
    let graphics_command_buffer = shared.graphics_command_buffers[shared.frame_index];

    unsafe {
      shared
        .vk_device
        .wait_for_fences(&[in_flight_fence], true, u64::MAX)
        .unwrap();
    }

    let transfer_command_buffer = shared
      .round_rect_sync
      .sync_to(&mut shared.model_buffer, &shared.vk_device);

    if let Some(transfer_command_buffer) = transfer_command_buffer {
      let queue_submit_info = vk::SubmitInfo {
        command_buffer_count: 1,
        p_command_buffers: &raw const transfer_command_buffer,
        signal_semaphore_count: 1,
        p_signal_semaphores: &raw const transfer_done_semaphore,
        ..Default::default()
      };

      unsafe {
        shared
          .vk_device
          .queue_submit(
            shared.transfer_queue,
            &[queue_submit_info],
            vk::Fence::null(),
          )
          .unwrap();
      }
    }

    let acquire_next_image_info = vk::AcquireNextImageInfoKHR {
      swapchain: state.swapchain,
      timeout: u64::MAX,
      semaphore: image_avail_semaphore,
      device_mask: 1,
      ..Default::default()
    };

    let swapchain_image_index = match unsafe {
      shared
        .vk_swapchain_device
        .acquire_next_image2(&acquire_next_image_info)
    } {
      Ok((swapchain_image_index, _swapchain_suboptimal)) => swapchain_image_index,
      Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
        let old_swapchain = state.swapchain;

        return match state.on_swapchain_suboptimal(&shared) {
          Ok(new_state) => Ok(Self {
            shared,
            state: new_state,
          }),
          Err(WindowMinimized) => Err(Renderer {
            shared,
            state: Creating { old_swapchain },
          }),
        };
      }
      Err(err) => panic!("{err}"),
    };

    let swapchain_framebuffer = state.swapchain_framebuffers[swapchain_image_index as usize];

    unsafe {
      shared
        .vk_device
        .reset_command_pool(graphics_command_pool, vk::CommandPoolResetFlags::empty())
        .unwrap();
    }

    let graphics_command_buffer_begin_info = vk::CommandBufferBeginInfo {
      flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    unsafe {
      shared
        .vk_device
        .begin_command_buffer(graphics_command_buffer, &graphics_command_buffer_begin_info)
        .unwrap();
    }

    let clear_values = [
      vk::ClearValue {
        color: vk::ClearColorValue {
          float32: [0.0, 0.0, 0.0, 1.0],
        },
      },
      vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
          depth: 1.0,
          ..Default::default()
        },
      },
    ];

    let render_pass_begin_info = vk::RenderPassBeginInfo {
      render_pass: shared.render_pass,
      framebuffer: swapchain_framebuffer,
      render_area: vk::Rect2D {
        extent: state.swapchain_extent,
        ..Default::default()
      },
      clear_value_count: clear_values.len().try_into().unwrap(),
      p_clear_values: clear_values.as_ptr(),
      ..Default::default()
    };

    let subpass_begin_info = vk::SubpassBeginInfo {
      contents: vk::SubpassContents::INLINE,
      ..Default::default()
    };

    unsafe {
      shared.vk_device.cmd_begin_render_pass2(
        graphics_command_buffer,
        &render_pass_begin_info,
        &subpass_begin_info,
      );
    }

    unsafe {
      shared.vk_device.cmd_bind_pipeline(
        graphics_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        shared.graphics_pipeline,
      );
    }

    let viewports = [vk::Viewport {
      width: state.swapchain_extent.width as f32,
      height: state.swapchain_extent.height as f32,
      min_depth: 0.0,
      max_depth: 1.0,
      ..Default::default()
    }];

    unsafe {
      shared
        .vk_device
        .cmd_set_viewport(graphics_command_buffer, 0, &viewports);
    }

    let scissors = [vk::Rect2D {
      extent: state.swapchain_extent,
      ..Default::default()
    }];

    unsafe {
      shared
        .vk_device
        .cmd_set_scissor(graphics_command_buffer, 0, &scissors);
    }

    let LogicalSize {
      width: cam_width,
      height: cam_height,
    } = shared
      .window
      .inner_size()
      .to_logical(shared.window.scale_factor());

    let push_consts = PushConsts {
      round_rect_buffer: shared.model_buffer.calc_read_addr(),
      cam_size: (cam_width, cam_height),
    };

    let raw_push_consts = unsafe {
      slice::from_raw_parts(
        (&raw const push_consts).cast(),
        mem::size_of::<PushConsts>(),
      )
    };

    unsafe {
      shared.vk_device.cmd_push_constants(
        graphics_command_buffer,
        shared.pipeline_layout,
        vk::ShaderStageFlags::VERTEX,
        0,
        raw_push_consts,
      );
    }

    let vertex_count = (shared.round_rect_sync.get_model_count() * RoundRect::get_vertex_count())
      .try_into()
      .unwrap();

    unsafe {
      shared
        .vk_device
        .cmd_draw(graphics_command_buffer, vertex_count, 1, 0, 0);
    }

    let subpass_end_info = vk::SubpassEndInfo::default();

    unsafe {
      shared
        .vk_device
        .cmd_end_render_pass2(graphics_command_buffer, &subpass_end_info);
    }

    unsafe {
      shared
        .vk_device
        .end_command_buffer(graphics_command_buffer)
        .unwrap();
    }

    let mut wait_semaphores = vec![image_avail_semaphore];
    let mut wait_dst_stage_masks = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

    if transfer_command_buffer.is_some() {
      wait_semaphores.push(transfer_done_semaphore);
      wait_dst_stage_masks.push(vk::PipelineStageFlags::VERTEX_SHADER);
    }

    let wait_semaphores = wait_semaphores;
    let wait_dst_stage_masks = wait_dst_stage_masks;

    let queue_submit_info = vk::SubmitInfo {
      wait_semaphore_count: wait_semaphores.len().try_into().unwrap(),
      p_wait_semaphores: wait_semaphores.as_ptr(),
      p_wait_dst_stage_mask: wait_dst_stage_masks.as_ptr(),
      command_buffer_count: 1,
      p_command_buffers: &raw const graphics_command_buffer,
      signal_semaphore_count: 1,
      p_signal_semaphores: &raw const render_done_semaphore,
      ..Default::default()
    };

    unsafe {
      shared.vk_device.reset_fences(&[in_flight_fence]).unwrap();
    }

    unsafe {
      shared
        .vk_device
        .queue_submit(shared.graphics_queue, &[queue_submit_info], in_flight_fence)
        .unwrap();
    }

    shared.window.pre_present_notify();

    let present_info = vk::PresentInfoKHR {
      wait_semaphore_count: 1,
      p_wait_semaphores: &raw const render_done_semaphore,
      swapchain_count: 1,
      p_swapchains: &raw const state.swapchain,
      p_image_indices: &raw const swapchain_image_index,
      ..Default::default()
    };

    match unsafe {
      shared
        .vk_swapchain_device
        .queue_present(shared.present_queue, &present_info)
    } {
      Ok(false) => (),
      Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
        let old_swapchain = state.swapchain;

        return match state.on_swapchain_suboptimal(&shared) {
          Ok(new_state) => Ok(Self {
            shared,
            state: new_state,
          }),
          Err(WindowMinimized) => Err(Renderer {
            shared,
            state: Creating { old_swapchain },
          }),
        };
      }
      Err(err) => panic!("{err}"),
    }

    Ok(Self {
      shared: Shared {
        frame_index: (shared.frame_index + 1) % consts::MAX_IN_FLIGHT_FRAME_COUNT,
        ..shared
      },
      state,
    })
  }

  pub(super) fn drop(self) {
    let Self { shared, state } = self;

    unsafe {
      shared.vk_device.device_wait_idle().unwrap();
    }

    state.drop(&shared, false);
    shared.drop();
  }
}

impl<State> Renderer<State> {
  #[inline]
  pub(super) const fn get_window(&self) -> &Window {
    &self.shared.window
  }

  #[inline]
  pub(super) const fn get_round_rect_sync(&mut self) -> &mut ModelSync<RoundRect> {
    &mut self.shared.round_rect_sync
  }
}
