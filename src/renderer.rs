use crate::pipelines::{
  RectPipeline,
  rect_pipeline::{self, RectPushConsts},
};
use ash::vk::{self, Handle};
use rustc_hash::FxHashSet;
use sdl3::video::Window;
use std::{ffi::CString, iter, mem, ptr, slice};

const MAX_IN_FLIGHT_FRAME_COUNT: usize = 2;

pub(super) enum AnyRenderer {
  Creating(Renderer<Creating>),
  Created(Renderer<Created>),
}

pub(super) enum FinishError {
  WindowMinimized(Box<Renderer<Creating>>),
}

pub(super) struct Creating {
  rect_pipeline: RectPipeline<rect_pipeline::Creating>,
}

pub(super) struct Created {
  swapchain_image_extent: vk::Extent2D,
  swapchain: vk::SwapchainKHR,
  _swapchain_images: Vec<vk::Image>,
  swapchain_image_views: Box<[vk::ImageView]>,
  framebuffers: Box<[vk::Framebuffer]>,
  rect_pipeline: RectPipeline<rect_pipeline::Created>,
  frame_index: usize,
}

pub(super) struct Renderer<State> {
  ash_entry: ash::Entry,
  vk_inst: ash::Instance,
  vk_surface: vk::SurfaceKHR,
  surface_inst: ash::khr::surface::Instance,
  vk_physical_device: vk::PhysicalDevice,
  graphics_queue_family_index: u32,
  present_queue_family_index: u32,
  vk_device: ash::Device,
  vk_allocator: vk_mem::Allocator,
  graphics_queue: vk::Queue,
  present_queue: vk::Queue,
  swapchain_device: ash::khr::swapchain::Device,
  vk_surface_format: vk::SurfaceFormatKHR,
  render_pass: vk::RenderPass,
  graphics_command_pools: Vec<vk::CommandPool>,
  graphics_command_buffers: Vec<vk::CommandBuffer>,
  image_avail_semaphores: Vec<vk::Semaphore>,
  render_done_semaphores: Vec<vk::Semaphore>,
  in_flight_fences: Vec<vk::Fence>,
  pipeline_cache: vk::PipelineCache,
  state: State,
}

impl Renderer<Creating> {
  pub(super) fn new(window: Window) -> Self {
    let ash_entry = unsafe { ash::Entry::load().unwrap() };
    let app_name = CString::new(window.title()).unwrap();

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
                Some(queue_family_index as _)
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
                  Some(queue_family_index as _)
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
          queue_family_index,
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

    let mut vk_allocator_create_info =
      vk_mem::AllocatorCreateInfo::new(&vk_inst, &vk_device, vk_physical_device);

    vk_allocator_create_info.flags = vk_mem::AllocatorCreateFlags::EXTERNALLY_SYNCHRONIZED
      | vk_mem::AllocatorCreateFlags::KHR_DEDICATED_ALLOCATION
      | vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;

    vk_allocator_create_info.preferred_large_heap_block_size = 1024 * 1024; // 1 MB
    vk_allocator_create_info.vulkan_api_version = vk::API_VERSION_1_3;

    let vk_allocator = unsafe { vk_mem::Allocator::new(vk_allocator_create_info).unwrap() };

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

    let graphics_queue = unsafe { vk_device.get_device_queue2(&graphics_queue_info) };
    let present_queue = unsafe { vk_device.get_device_queue2(&present_queue_info) };

    let swapchain_device = ash::khr::swapchain::Device::new(&vk_inst, &vk_device);

    let vk_surface_formats = unsafe {
      surface_inst
        .get_physical_device_surface_formats(vk_physical_device, vk_surface)
        .unwrap()
    };

    let vk_surface_format = *vk_surface_formats
      .iter()
      .find(|&surface_format| {
        surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
          && (surface_format.format == vk::Format::B8G8R8A8_SRGB
            || surface_format.format == vk::Format::R8G8B8A8_SRGB)
      })
      .unwrap_or(&vk_surface_formats[0]);

    let attachment_descs = [vk::AttachmentDescription2 {
      format: vk_surface_format.format,
      samples: vk::SampleCountFlags::TYPE_1,
      load_op: vk::AttachmentLoadOp::CLEAR,
      store_op: vk::AttachmentStoreOp::STORE,
      stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
      stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
      initial_layout: vk::ImageLayout::UNDEFINED,
      final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
      ..Default::default()
    }];

    let color_attachment_refs = [vk::AttachmentReference2 {
      attachment: 0,
      layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
      ..Default::default()
    }];

    let subpass_descs = [vk::SubpassDescription2 {
      pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
      color_attachment_count: color_attachment_refs.len() as _,
      p_color_attachments: color_attachment_refs.as_ptr(),
      ..Default::default()
    }];

    let subpass_deps = [vk::SubpassDependency2 {
      src_subpass: vk::SUBPASS_EXTERNAL,
      dst_subpass: 0,
      src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
      dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
      src_access_mask: vk::AccessFlags::empty(),
      dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
      dependency_flags: vk::DependencyFlags::BY_REGION,
      ..Default::default()
    }];

    let render_pass_create_info = vk::RenderPassCreateInfo2 {
      attachment_count: attachment_descs.len() as _,
      p_attachments: attachment_descs.as_ptr(),
      subpass_count: subpass_descs.len() as _,
      p_subpasses: subpass_descs.as_ptr(),
      dependency_count: subpass_deps.len() as _,
      p_dependencies: subpass_deps.as_ptr(),
      ..Default::default()
    };

    let render_pass = unsafe {
      vk_device
        .create_render_pass2(&render_pass_create_info, None)
        .unwrap()
    };

    let graphics_command_pool_create_info = vk::CommandPoolCreateInfo {
      flags: vk::CommandPoolCreateFlags::TRANSIENT,
      queue_family_index: graphics_queue_family_index,
      ..Default::default()
    };

    let image_avail_semaphore_create_info = vk::SemaphoreCreateInfo::default();
    let render_done_semaphore_create_info = vk::SemaphoreCreateInfo::default();

    let in_flight_fence_create_info = vk::FenceCreateInfo {
      flags: vk::FenceCreateFlags::SIGNALED,
      ..Default::default()
    };

    let (graphics_command_pools, graphics_command_buffers): (Vec<_>, Vec<_>) = (0
      ..MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|_| unsafe {
        let command_pool = vk_device
          .create_command_pool(&graphics_command_pool_create_info, None)
          .unwrap();

        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
          command_pool,
          level: vk::CommandBufferLevel::PRIMARY,
          command_buffer_count: 1,
          ..Default::default()
        };

        let command_buffer = vk_device
          .allocate_command_buffers(&command_buffer_alloc_info)
          .unwrap()[0];

        (command_pool, command_buffer)
      })
      .unzip();

    let (image_avail_semaphores, (render_done_semaphores, in_flight_fences)): (
      Vec<_>,
      (Vec<_>, Vec<_>),
    ) = (0..MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|_| unsafe {
        let image_avail_semaphore = vk_device
          .create_semaphore(&image_avail_semaphore_create_info, None)
          .unwrap();

        let render_done_semaphore = vk_device
          .create_semaphore(&render_done_semaphore_create_info, None)
          .unwrap();

        let in_flight_fence = vk_device
          .create_fence(&in_flight_fence_create_info, None)
          .unwrap();

        (
          image_avail_semaphore,
          (render_done_semaphore, in_flight_fence),
        )
      })
      .unzip();

    let pipeline_cache_create_info = vk::PipelineCacheCreateInfo::default();

    let pipeline_cache = unsafe {
      vk_device
        .create_pipeline_cache(&pipeline_cache_create_info, None)
        .unwrap_or_else(|err| {
          println!("Failed to create pipeline cache: {err}");
          vk::PipelineCache::null()
        })
    };

    let rect_pipeline = RectPipeline::new(&vk_device);

    Self {
      ash_entry,
      vk_inst,
      vk_surface,
      surface_inst,
      vk_physical_device,
      graphics_queue_family_index,
      present_queue_family_index,
      vk_device,
      vk_allocator,
      graphics_queue,
      present_queue,
      swapchain_device,
      vk_surface_format,
      render_pass,
      graphics_command_pools,
      graphics_command_buffers,
      image_avail_semaphores,
      render_done_semaphores,
      in_flight_fences,
      pipeline_cache,
      state: Creating { rect_pipeline },
    }
  }

  pub(super) fn finish(self, window: Window) -> Result<Renderer<Created>, FinishError> {
    let vk_surface_caps = unsafe {
      self
        .surface_inst
        .get_physical_device_surface_capabilities(self.vk_physical_device, self.vk_surface)
        .unwrap()
    };

    let window_size = window.size_in_pixels();

    let swapchain_image_extent = if vk_surface_caps.current_extent.width != u32::MAX {
      vk_surface_caps.current_extent
    } else {
      vk::Extent2D {
        width: window_size.0.clamp(
          vk_surface_caps.min_image_extent.width,
          vk_surface_caps.max_image_extent.width,
        ),
        height: window_size.1.clamp(
          vk_surface_caps.min_image_extent.height,
          vk_surface_caps.max_image_extent.height,
        ),
      }
    };

    if swapchain_image_extent.width == 0 || swapchain_image_extent.height == 0 {
      return Err(FinishError::WindowMinimized(Box::new(self)));
    }

    let swapchain_image_count = vk_surface_caps.min_image_count + 1;

    let swapchain_image_count = if vk_surface_caps.max_image_count > 0 {
      swapchain_image_count.min(vk_surface_caps.max_image_count)
    } else {
      swapchain_image_count
    };

    let queue_family_indices = [
      self.graphics_queue_family_index,
      self.present_queue_family_index,
    ];

    let (
      swapchain_image_sharing_mode,
      swapchain_queue_family_index_count,
      swapchain_queue_family_indices,
    ) = if self.graphics_queue_family_index == self.present_queue_family_index {
      (vk::SharingMode::EXCLUSIVE, 0, ptr::null())
    } else {
      (
        vk::SharingMode::CONCURRENT,
        queue_family_indices.len() as _,
        queue_family_indices.as_ptr(),
      )
    };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
      surface: self.vk_surface,
      min_image_count: swapchain_image_count,
      image_format: self.vk_surface_format.format,
      image_color_space: self.vk_surface_format.color_space,
      image_extent: swapchain_image_extent,
      image_array_layers: 1,
      image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
      image_sharing_mode: swapchain_image_sharing_mode,
      queue_family_index_count: swapchain_queue_family_index_count,
      p_queue_family_indices: swapchain_queue_family_indices,
      pre_transform: vk_surface_caps.current_transform,
      composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
      present_mode: vk::PresentModeKHR::FIFO,
      clipped: vk::TRUE,
      ..Default::default()
    };

    let swapchain = unsafe {
      self
        .swapchain_device
        .create_swapchain(&swapchain_create_info, None)
        .unwrap()
    };

    let swapchain_images = unsafe {
      self
        .swapchain_device
        .get_swapchain_images(swapchain)
        .unwrap()
    };

    let swapchain_image_views = swapchain_images
      .iter()
      .map(|&image| {
        let image_view_create_info = vk::ImageViewCreateInfo {
          image,
          view_type: vk::ImageViewType::TYPE_2D,
          format: self.vk_surface_format.format,
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
          self
            .vk_device
            .create_image_view(&image_view_create_info, None)
            .unwrap()
        }
      })
      .collect::<Box<_>>();

    let framebuffers = swapchain_image_views
      .iter()
      .map(|&image_view| {
        let framebuffer_create_info = vk::FramebufferCreateInfo {
          render_pass: self.render_pass,
          attachment_count: 1,
          p_attachments: &image_view,
          width: swapchain_image_extent.width,
          height: swapchain_image_extent.height,
          layers: 1,
          ..Default::default()
        };

        unsafe {
          self
            .vk_device
            .create_framebuffer(&framebuffer_create_info, None)
            .unwrap()
        }
      })
      .collect::<Box<_>>();

    let rect_pipeline = self.state.rect_pipeline.finish(
      &self.vk_device,
      self.render_pass,
      self.pipeline_cache,
      swapchain_image_extent,
    );

    Ok(Renderer {
      ash_entry: self.ash_entry,
      vk_inst: self.vk_inst,
      vk_surface: self.vk_surface,
      surface_inst: self.surface_inst,
      vk_physical_device: self.vk_physical_device,
      graphics_queue_family_index: self.graphics_queue_family_index,
      present_queue_family_index: self.present_queue_family_index,
      vk_device: self.vk_device,
      vk_allocator: self.vk_allocator,
      graphics_queue: self.graphics_queue,
      present_queue: self.present_queue,
      swapchain_device: self.swapchain_device,
      vk_surface_format: self.vk_surface_format,
      render_pass: self.render_pass,
      graphics_command_pools: self.graphics_command_pools,
      graphics_command_buffers: self.graphics_command_buffers,
      image_avail_semaphores: self.image_avail_semaphores,
      render_done_semaphores: self.render_done_semaphores,
      in_flight_fences: self.in_flight_fences,
      pipeline_cache: self.pipeline_cache,
      state: Created {
        swapchain_image_extent,
        swapchain,
        _swapchain_images: swapchain_images,
        swapchain_image_views,
        framebuffers,
        rect_pipeline,
        frame_index: 0,
      },
    })
  }

  pub(super) fn drop(self) {
    unsafe {
      self.vk_device.device_wait_idle().unwrap();
      self.state.rect_pipeline.drop(&self.vk_device);

      self
        .vk_device
        .destroy_pipeline_cache(self.pipeline_cache, None);

      self
        .in_flight_fences
        .iter()
        .for_each(|&fence| self.vk_device.destroy_fence(fence, None));

      self
        .render_done_semaphores
        .iter()
        .for_each(|&semaphore| self.vk_device.destroy_semaphore(semaphore, None));

      self
        .image_avail_semaphores
        .iter()
        .for_each(|&semaphore| self.vk_device.destroy_semaphore(semaphore, None));

      self
        .graphics_command_pools
        .iter()
        .for_each(|&command_pool| self.vk_device.destroy_command_pool(command_pool, None));

      self.vk_device.destroy_render_pass(self.render_pass, None);
      drop(self.vk_allocator);
      self.vk_device.destroy_device(None);
      self.surface_inst.destroy_surface(self.vk_surface, None);
      self.vk_inst.destroy_instance(None);
    }
  }
}

impl Renderer<Created> {
  pub(super) fn render(self) -> AnyRenderer {
    let graphics_command_pool = self.graphics_command_pools[self.state.frame_index];
    let graphics_command_buffer = self.graphics_command_buffers[self.state.frame_index];
    let image_avail_semaphore = self.image_avail_semaphores[self.state.frame_index];
    let render_done_semaphore = self.render_done_semaphores[self.state.frame_index];
    let in_flight_fence = self.in_flight_fences[self.state.frame_index];

    unsafe {
      self
        .vk_device
        .wait_for_fences(&[in_flight_fence], true, u64::MAX)
        .unwrap();

      let acquire_next_image_info = vk::AcquireNextImageInfoKHR {
        swapchain: self.state.swapchain,
        timeout: u64::MAX,
        semaphore: image_avail_semaphore,
        device_mask: 1,
        ..Default::default()
      };

      let swapchain_image_index = match self
        .swapchain_device
        .acquire_next_image2(&acquire_next_image_info)
      {
        Ok((swapchain_image_index, _)) => swapchain_image_index,
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
          return AnyRenderer::Creating(self.on_swapchain_suboptimal());
        }
        Err(err) => panic!("{err}"),
      };

      self
        .vk_device
        .reset_command_pool(graphics_command_pool, vk::CommandPoolResetFlags::empty())
        .unwrap();

      let graphics_command_buffer_begin_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
      };

      self
        .vk_device
        .begin_command_buffer(graphics_command_buffer, &graphics_command_buffer_begin_info)
        .unwrap();

      let clear_values = [vk::ClearValue {
        color: vk::ClearColorValue {
          float32: [0.0, 0.0, 0.0, 0.0],
        },
      }];

      let render_pass_begin_info = vk::RenderPassBeginInfo {
        render_pass: self.render_pass,
        framebuffer: self.state.framebuffers[swapchain_image_index as usize],
        render_area: vk::Rect2D {
          extent: self.state.swapchain_image_extent,
          ..Default::default()
        },
        clear_value_count: clear_values.len() as _,
        p_clear_values: clear_values.as_ptr(),
        ..Default::default()
      };

      let subpass_begin_info = vk::SubpassBeginInfo {
        contents: vk::SubpassContents::INLINE,
        ..Default::default()
      };

      self.vk_device.cmd_begin_render_pass2(
        graphics_command_buffer,
        &render_pass_begin_info,
        &subpass_begin_info,
      );

      self.vk_device.cmd_bind_pipeline(
        graphics_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.state.rect_pipeline.get_pipeline(),
      );

      let rect_push_consts = RectPushConsts {
        rect_buffer: 1,
        cam_size: (
          self.state.swapchain_image_extent.width as _,
          self.state.swapchain_image_extent.height as _,
        ),
      };

      self.vk_device.cmd_push_constants(
        graphics_command_buffer,
        self.state.rect_pipeline.get_pipeline_layout(),
        vk::ShaderStageFlags::VERTEX,
        0,
        slice::from_raw_parts(
          &rect_push_consts as *const _ as *const _,
          mem::size_of::<RectPushConsts>(),
        ),
      );

      self.vk_device.cmd_draw(graphics_command_buffer, 6, 1, 0, 0);
      let subpass_end_info = vk::SubpassEndInfo::default();

      self
        .vk_device
        .cmd_end_render_pass2(graphics_command_buffer, &subpass_end_info);

      self
        .vk_device
        .end_command_buffer(graphics_command_buffer)
        .unwrap();

      self.vk_device.reset_fences(&[in_flight_fence]).unwrap();

      let wait_dst_stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;

      let graphics_queue_submit_info = vk::SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_avail_semaphore,
        p_wait_dst_stage_mask: &wait_dst_stage_mask,
        command_buffer_count: 1,
        p_command_buffers: &graphics_command_buffer,
        signal_semaphore_count: 1,
        p_signal_semaphores: &render_done_semaphore,
        ..Default::default()
      };

      self
        .vk_device
        .queue_submit(
          self.graphics_queue,
          &[graphics_queue_submit_info],
          in_flight_fence,
        )
        .unwrap();

      let present_info = vk::PresentInfoKHR {
        wait_semaphore_count: 1,
        p_wait_semaphores: &render_done_semaphore,
        swapchain_count: 1,
        p_swapchains: &self.state.swapchain,
        p_image_indices: &swapchain_image_index,
        ..Default::default()
      };

      match self
        .swapchain_device
        .queue_present(self.present_queue, &present_info)
      {
        Ok(false) => {}
        Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
          return AnyRenderer::Creating(self.on_swapchain_suboptimal());
        }
        Err(err) => panic!("{err}"),
      }
    }

    AnyRenderer::Created(Self {
      state: Created {
        frame_index: (self.state.frame_index + 1) % MAX_IN_FLIGHT_FRAME_COUNT,
        ..self.state
      },
      ..self
    })
  }

  pub(super) fn on_swapchain_suboptimal(self) -> Renderer<Creating> {
    unsafe {
      self.vk_device.device_wait_idle().unwrap();
    }

    let rect_pipeline = self.state.rect_pipeline.on_swapchain_suboptimal();

    unsafe {
      self
        .state
        .framebuffers
        .iter()
        .for_each(|&framebuffer| self.vk_device.destroy_framebuffer(framebuffer, None));

      self
        .state
        .swapchain_image_views
        .iter()
        .for_each(|&image_view| self.vk_device.destroy_image_view(image_view, None));

      self
        .swapchain_device
        .destroy_swapchain(self.state.swapchain, None);
    }

    Renderer {
      ash_entry: self.ash_entry,
      vk_inst: self.vk_inst,
      vk_surface: self.vk_surface,
      surface_inst: self.surface_inst,
      vk_physical_device: self.vk_physical_device,
      graphics_queue_family_index: self.graphics_queue_family_index,
      present_queue_family_index: self.present_queue_family_index,
      vk_device: self.vk_device,
      vk_allocator: self.vk_allocator,
      graphics_queue: self.graphics_queue,
      present_queue: self.present_queue,
      swapchain_device: self.swapchain_device,
      vk_surface_format: self.vk_surface_format,
      render_pass: self.render_pass,
      graphics_command_pools: self.graphics_command_pools,
      graphics_command_buffers: self.graphics_command_buffers,
      image_avail_semaphores: self.image_avail_semaphores,
      render_done_semaphores: self.render_done_semaphores,
      in_flight_fences: self.in_flight_fences,
      pipeline_cache: self.pipeline_cache,
      state: Creating { rect_pipeline },
    }
  }

  pub(super) fn drop(self) {
    unsafe {
      self.vk_device.device_wait_idle().unwrap();
      self.state.rect_pipeline.drop(&self.vk_device);

      self
        .state
        .framebuffers
        .iter()
        .for_each(|&framebuffer| self.vk_device.destroy_framebuffer(framebuffer, None));

      self
        .state
        .swapchain_image_views
        .iter()
        .for_each(|&image_view| self.vk_device.destroy_image_view(image_view, None));

      self
        .swapchain_device
        .destroy_swapchain(self.state.swapchain, None);

      self
        .vk_device
        .destroy_pipeline_cache(self.pipeline_cache, None);

      self
        .in_flight_fences
        .iter()
        .for_each(|&fence| self.vk_device.destroy_fence(fence, None));

      self
        .render_done_semaphores
        .iter()
        .for_each(|&semaphore| self.vk_device.destroy_semaphore(semaphore, None));

      self
        .image_avail_semaphores
        .iter()
        .for_each(|&semaphore| self.vk_device.destroy_semaphore(semaphore, None));

      self
        .graphics_command_pools
        .iter()
        .for_each(|&command_pool| self.vk_device.destroy_command_pool(command_pool, None));

      self.vk_device.destroy_render_pass(self.render_pass, None);
      drop(self.vk_allocator);
      self.vk_device.destroy_device(None);
      self.surface_inst.destroy_surface(self.vk_surface, None);
      self.vk_inst.destroy_instance(None);
    }
  }
}
