use crate::{
  models::{AtlasSizes, ModelCapacities},
  pipelines::{
    glyph_pipeline::{Glyph, GlyphPushConsts},
    round_rect_pipeline::{RoundRect, RoundRectPushConsts},
  },
  renderers::{
    MAX_IN_FLIGHT_FRAME_COUNT, ModelRenderer, model_renderer,
    text_renderer::{self, TextRenderer},
  },
};
use ash::vk::{self, Handle};
use rustc_hash::FxHashSet;
use sdl3::video::Window;
use std::{
  ffi::{CStr, CString, c_void},
  iter, mem, ptr,
};
use vk_mem::Alloc;

pub(crate) enum AnyRenderer {
  Creating(Renderer<Creating>),
  Created(Renderer<Created>),
}

pub(crate) enum FinishError {
  WindowMinimized(Box<Renderer<Creating>>),
}

pub(crate) struct Creating {
  text_renderer: TextRenderer<text_renderer::Creating>,
  round_rect_renderer: ModelRenderer<model_renderer::Creating<RoundRect>>,
}

pub(crate) struct Created {
  swapchain_image_extent: vk::Extent2D,
  swapchain: vk::SwapchainKHR,
  _swapchain_images: Vec<vk::Image>,
  swapchain_image_views: Box<[vk::ImageView]>,
  msaa_color_image: vk::Image,
  msaa_color_image_alloc: Option<vk_mem::Allocation>,
  msaa_color_image_view: vk::ImageView,
  framebuffers: Box<[vk::Framebuffer]>,
  text_renderer: TextRenderer<text_renderer::Created>,
  round_rect_renderer: ModelRenderer<model_renderer::Created<RoundRect>>,
  frame_index: usize,
}

pub(crate) struct Renderer<State> {
  ash_entry: ash::Entry,
  vk_inst: ash::Instance,
  vk_surface: vk::SurfaceKHR,
  surface_inst: ash::khr::surface::Instance,
  vk_physical_device: vk::PhysicalDevice,
  graphics_queue_family_index: u32,
  present_queue_family_index: u32,
  transfer_queue_family_index: u32,
  vk_device: ash::Device,
  graphics_queue: vk::Queue,
  present_queue: vk::Queue,
  transfer_queue: vk::Queue,
  vk_allocator: vk_mem::Allocator,
  model_buffer: vk::Buffer,
  model_buffer_alloc: vk_mem::Allocation,
  model_buffer_addr: vk::DeviceAddress,
  model_buffer_data: *mut c_void,
  swapchain_device: ash::khr::swapchain::Device,
  vk_surface_format: vk::SurfaceFormatKHR,
  render_pass: vk::RenderPass,
  graphics_command_pools: Vec<vk::CommandPool>,
  graphics_command_buffers: Vec<vk::CommandBuffer>,
  transition_command_pool: vk::CommandPool,
  transition_command_buffer: vk::CommandBuffer,
  transfer_command_pools: Vec<vk::CommandPool>,
  transfer_command_buffers: Vec<vk::CommandBuffer>,
  descriptor_pool: vk::DescriptorPool,
  image_avail_semaphores: Vec<vk::Semaphore>,
  render_done_semaphores: Vec<vk::Semaphore>,
  in_flight_fences: Vec<vk::Fence>,
  transition_done_semaphore: vk::Semaphore,
  glyph_atlas_semaphores: Vec<(vk::Semaphore, u64)>,
  read_atlas_semaphores: Vec<(vk::Semaphore, u64)>,
  pipeline_cache: vk::PipelineCache,
  msaa_samples: vk::SampleCountFlags,
  atlas_sizes: AtlasSizes,
  cam_position: Option<(f32, f32)>,
  cam_size: Option<(f32, f32)>,
  state: State,
}

impl Renderer<Creating> {
  pub(crate) fn new(
    window: Window,
    model_capacities: ModelCapacities,
    atlas_sizes: AtlasSizes,
  ) -> Self {
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
      .into_iter()
      .map(|ext_name| CString::new(ext_name).unwrap())
      .collect::<Box<_>>();

    let vk_ext_names = vk_ext_names
      .iter()
      .map(|ext_name| ext_name.as_ptr())
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

    let (
      vk_physical_device,
      vk_physical_device_props,
      graphics_queue_family_index,
      present_queue_family_index,
      transfer_queue_family_index,
    ) = vk_physical_devices
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

        let transfer_queue_family_index = queue_family_props.iter().enumerate().find_map(
          |(queue_family_index, queue_family_props)| {
            if queue_family_props
              .queue_family_properties
              .queue_flags
              .contains(vk::QueueFlags::TRANSFER)
            {
              Some(queue_family_index as _)
            } else {
              None
            }
          },
        );

        let mut vk_physical_device_props = vk::PhysicalDeviceProperties2::default();

        unsafe {
          vk_inst
            .get_physical_device_properties2(vk_physical_device, &mut vk_physical_device_props);
        }

        if let (
          Some(graphics_queue_family_index),
          Some(present_queue_family_index),
          Some(transfer_queue_family_index),
        ) = (
          graphics_queue_family_index,
          present_queue_family_index,
          transfer_queue_family_index,
        ) {
          Some((
            vk_physical_device,
            vk_physical_device_props,
            graphics_queue_family_index,
            present_queue_family_index,
            transfer_queue_family_index,
          ))
        } else {
          None
        }
      })
      .max_by_key(|&(_, vk_physical_device_props, _, _, _)| {
        match vk_physical_device_props.properties.device_type {
          vk::PhysicalDeviceType::INTEGRATED_GPU => 4,
          vk::PhysicalDeviceType::DISCRETE_GPU => 3,
          vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
          vk::PhysicalDeviceType::CPU => 1,
          _ => 0,
        }
      })
      .unwrap();

    let framebuffer_sample_counts = vk_physical_device_props
      .properties
      .limits
      .framebuffer_color_sample_counts;

    let msaa_samples = if framebuffer_sample_counts.contains(vk::SampleCountFlags::TYPE_2) {
      vk::SampleCountFlags::TYPE_2
    } else {
      println!("MSAA not supported");
      vk::SampleCountFlags::TYPE_1
    };

    let queue_priorities = [1.0];

    let queue_create_infos = FxHashSet::from_iter([
      graphics_queue_family_index,
      present_queue_family_index,
      transfer_queue_family_index,
    ])
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

    const REQ_VK_DEVICE_EXT_NAMES: &[&CStr] = &[
      vk::KHR_SWAPCHAIN_NAME,
      vk::EXT_PAGEABLE_DEVICE_LOCAL_MEMORY_NAME,
      vk::EXT_MEMORY_PRIORITY_NAME,
      vk::KHR_PORTABILITY_SUBSET_NAME,
    ];

    let vk_device_ext_name_c_strs = vk_ext_props
      .iter()
      .filter_map(|vk_ext_props| {
        let vk_ext_name = vk_ext_props.extension_name_as_c_str().unwrap();

        if REQ_VK_DEVICE_EXT_NAMES.contains(&vk_ext_name) {
          Some(vk_ext_name)
        } else {
          None
        }
      })
      .collect::<Box<_>>();

    let vk_device_ext_names = vk_device_ext_name_c_strs
      .iter()
      .map(|&ext_name_c_str| ext_name_c_str.as_ptr())
      .collect::<Box<_>>();

    let vk_physical_device_pageable_device_local_memory_features =
      vk::PhysicalDevicePageableDeviceLocalMemoryFeaturesEXT {
        pageable_device_local_memory: vk::TRUE,
        ..Default::default()
      };

    let vk_physical_device_vulkan_12_features = vk::PhysicalDeviceVulkan12Features {
      buffer_device_address: vk::TRUE,
      scalar_block_layout: vk::TRUE,
      storage_buffer8_bit_access: vk::TRUE,
      timeline_semaphore: vk::TRUE,
      vulkan_memory_model: vk::TRUE,
      vulkan_memory_model_device_scope: vk::TRUE,
      uniform_and_storage_buffer8_bit_access: vk::TRUE,
      p_next: if vk_device_ext_name_c_strs.contains(&vk::EXT_PAGEABLE_DEVICE_LOCAL_MEMORY_NAME) {
        &vk_physical_device_pageable_device_local_memory_features as *const _ as *mut _
      } else {
        ptr::null_mut()
      },
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
    let total_model_capacity_size = model_capacities.calc_total_size();

    let mut vk_allocator_create_info =
      vk_mem::AllocatorCreateInfo::new(&vk_inst, &vk_device, vk_physical_device);

    vk_allocator_create_info.flags = vk_mem::AllocatorCreateFlags::EXTERNALLY_SYNCHRONIZED
      | vk_mem::AllocatorCreateFlags::KHR_DEDICATED_ALLOCATION
      | vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS
      | vk_mem::AllocatorCreateFlags::EXT_MEMORY_PRIORITY;

    vk_allocator_create_info.preferred_large_heap_block_size =
      (MAX_IN_FLIGHT_FRAME_COUNT * total_model_capacity_size).max(1024 * 1024) as _; // 1 MB

    vk_allocator_create_info.vulkan_api_version = vk::API_VERSION_1_3;

    let vk_allocator = unsafe { vk_mem::Allocator::new(vk_allocator_create_info).unwrap() };

    let model_buffer_create_info = vk::BufferCreateInfo {
      size: (MAX_IN_FLIGHT_FRAME_COUNT * total_model_capacity_size) as _,
      usage: vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let model_buffer_alloc_create_info = vk_mem::AllocationCreateInfo {
      flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
        | vk_mem::AllocationCreateFlags::MAPPED,
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      ..Default::default()
    };

    let (model_buffer, model_buffer_alloc) = unsafe {
      vk_allocator
        .create_buffer(&model_buffer_create_info, &model_buffer_alloc_create_info)
        .unwrap()
    };

    let model_buffer_device_addr_info = vk::BufferDeviceAddressInfo {
      buffer: model_buffer,
      ..Default::default()
    };

    let model_buffer_addr =
      unsafe { vk_device.get_buffer_device_address(&model_buffer_device_addr_info) };

    let model_buffer_alloc_info = vk_allocator.get_allocation_info(&model_buffer_alloc);
    let model_buffer_data = model_buffer_alloc_info.mapped_data;

    let swapchain_device = ash::khr::swapchain::Device::new(&vk_inst, &vk_device);

    let vk_surface_formats = unsafe {
      surface_inst
        .get_physical_device_surface_formats(vk_physical_device, vk_surface)
        .unwrap()
    };

    let vk_surface_format = *vk_surface_formats
      .iter()
      .find(|&surface_format| {
        surface_format.format == vk::Format::B8G8R8A8_UNORM
          || surface_format.format == vk::Format::R8G8B8A8_UNORM
      })
      .unwrap_or(&vk_surface_formats[0]);

    let mut attachment_descs = vec![];
    let mut color_attachment_refs = vec![];
    let mut resolve_attachment_refs = vec![];

    if msaa_samples > vk::SampleCountFlags::TYPE_1 {
      attachment_descs.extend([
        // Attachment 0: MSAA color attachment
        vk::AttachmentDescription2 {
          format: vk_surface_format.format,
          samples: msaa_samples,
          load_op: vk::AttachmentLoadOp::CLEAR,
          store_op: vk::AttachmentStoreOp::DONT_CARE,
          stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
          stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
          initial_layout: vk::ImageLayout::UNDEFINED,
          final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          ..Default::default()
        },
        // Attachment 1: Resolve attachment (swapchain image)
        vk::AttachmentDescription2 {
          format: vk_surface_format.format,
          samples: vk::SampleCountFlags::TYPE_1,
          load_op: vk::AttachmentLoadOp::DONT_CARE,
          store_op: vk::AttachmentStoreOp::STORE,
          stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
          stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
          initial_layout: vk::ImageLayout::UNDEFINED,
          final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
          ..Default::default()
        },
      ]);

      // Attachment 0: MSAA color attachment reference
      color_attachment_refs.push(vk::AttachmentReference2 {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
      });

      // Attachment 1: Resolve attachment reference (swapchain image)
      resolve_attachment_refs.push(vk::AttachmentReference2 {
        attachment: 1,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
      });
    } else {
      // Attachment 0: swapchain color attachment
      attachment_descs.push(vk::AttachmentDescription2 {
        format: vk_surface_format.format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
      });

      // Attachment 0: swapchain color attachment reference
      color_attachment_refs.push(vk::AttachmentReference2 {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
      });
    }

    let subpass_descs = [vk::SubpassDescription2 {
      pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
      color_attachment_count: color_attachment_refs.len() as _,
      p_color_attachments: color_attachment_refs.as_ptr(),
      p_resolve_attachments: if msaa_samples > vk::SampleCountFlags::TYPE_1 {
        resolve_attachment_refs.as_ptr()
      } else {
        ptr::null()
      },
      ..Default::default()
    }];

    let subpass_deps = [vk::SubpassDependency2 {
      src_subpass: vk::SUBPASS_EXTERNAL,
      dst_subpass: 0,
      src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
      dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
      src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
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

    let transition_command_pool_create_info = vk::CommandPoolCreateInfo {
      queue_family_index: transfer_queue_family_index,
      ..Default::default()
    };

    let transfer_command_pool_create_info = vk::CommandPoolCreateInfo {
      queue_family_index: transfer_queue_family_index,
      ..Default::default()
    };

    let image_avail_semaphore_create_info = vk::SemaphoreCreateInfo::default();
    let render_done_semaphore_create_info = vk::SemaphoreCreateInfo::default();

    let in_flight_fence_create_info = vk::FenceCreateInfo {
      flags: vk::FenceCreateFlags::SIGNALED,
      ..Default::default()
    };

    let transition_done_semaphore_type_create_info = vk::SemaphoreTypeCreateInfo {
      semaphore_type: vk::SemaphoreType::TIMELINE,
      initial_value: 0,
      ..Default::default()
    };

    let transition_done_semaphore_create_info = vk::SemaphoreCreateInfo {
      p_next: &transition_done_semaphore_type_create_info as *const _ as *const _,
      ..Default::default()
    };

    let glyph_atlas_semaphore_type_create_info = vk::SemaphoreTypeCreateInfo {
      semaphore_type: vk::SemaphoreType::TIMELINE,
      initial_value: 0,
      ..Default::default()
    };

    let glyph_atlas_semaphore_create_info = vk::SemaphoreCreateInfo {
      p_next: &glyph_atlas_semaphore_type_create_info as *const _ as *const _,
      ..Default::default()
    };

    let read_atlas_semaphore_type_create_info = vk::SemaphoreTypeCreateInfo {
      semaphore_type: vk::SemaphoreType::TIMELINE,
      initial_value: 0,
      ..Default::default()
    };

    let read_atlas_semaphore_create_info = vk::SemaphoreCreateInfo {
      p_next: &read_atlas_semaphore_type_create_info as *const _ as *const _,
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

    let transition_command_pool = unsafe {
      vk_device
        .create_command_pool(&transition_command_pool_create_info, None)
        .unwrap()
    };

    let transition_command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
      command_pool: transition_command_pool,
      level: vk::CommandBufferLevel::PRIMARY,
      command_buffer_count: 1,
      ..Default::default()
    };

    let transition_command_buffer = unsafe {
      vk_device
        .allocate_command_buffers(&transition_command_buffer_alloc_info)
        .unwrap()[0]
    };

    let (transfer_command_pools, transfer_command_buffers): (Vec<_>, Vec<_>) = (0
      ..MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|_| unsafe {
        let command_pool = vk_device
          .create_command_pool(&transfer_command_pool_create_info, None)
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

    let descriptor_pool_sizes = [vk::DescriptorPoolSize {
      ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      descriptor_count: MAX_IN_FLIGHT_FRAME_COUNT as _,
    }];

    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
      max_sets: MAX_IN_FLIGHT_FRAME_COUNT as _,
      pool_size_count: descriptor_pool_sizes.len() as _,
      p_pool_sizes: descriptor_pool_sizes.as_ptr(),
      ..Default::default()
    };

    let descriptor_pool = unsafe {
      vk_device
        .create_descriptor_pool(&descriptor_pool_create_info, None)
        .unwrap()
    };

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

    let transition_done_semaphore = unsafe {
      vk_device
        .create_semaphore(&transition_done_semaphore_create_info, None)
        .unwrap()
    };

    let (glyph_atlas_semaphores, read_atlas_semaphores): (Vec<_>, Vec<_>) = (0
      ..MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|_| unsafe {
        let glyph_atlas_semaphore = vk_device
          .create_semaphore(&glyph_atlas_semaphore_create_info, None)
          .unwrap();

        let read_atlas_semaphore = vk_device
          .create_semaphore(&read_atlas_semaphore_create_info, None)
          .unwrap();

        ((glyph_atlas_semaphore, 0), (read_atlas_semaphore, 0))
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

    let transition_command_buffer_begin_info = vk::CommandBufferBeginInfo {
      flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    unsafe {
      vk_device
        .begin_command_buffer(
          transition_command_buffer,
          &transition_command_buffer_begin_info,
        )
        .unwrap();
    }
    let text_renderer = TextRenderer::new(
      window,
      &vk_device,
      &vk_allocator,
      transition_command_buffer,
      descriptor_pool,
      model_capacities.glyph_capacity,
      atlas_sizes.glyph_atlas_size,
    );

    let round_rect_renderer = ModelRenderer::new(&vk_device, model_capacities.round_rect_capacity);

    unsafe {
      vk_device
        .end_command_buffer(transition_command_buffer)
        .unwrap();
    }

    let transition_done_semaphore_value = 1;

    let transition_done_semaphore_submit_info = vk::TimelineSemaphoreSubmitInfo {
      signal_semaphore_value_count: 1,
      p_signal_semaphore_values: &transition_done_semaphore_value,
      ..Default::default()
    };

    let transfer_queue_submit_info = vk::SubmitInfo {
      command_buffer_count: 1,
      p_command_buffers: &transition_command_buffer,
      signal_semaphore_count: 1,
      p_signal_semaphores: &transition_done_semaphore,
      p_next: &transition_done_semaphore_submit_info as *const _ as *const _,
      ..Default::default()
    };

    unsafe {
      vk_device
        .queue_submit(
          transfer_queue,
          &[transfer_queue_submit_info],
          vk::Fence::null(),
        )
        .unwrap();
    }

    Self {
      ash_entry,
      vk_inst,
      vk_surface,
      surface_inst,
      vk_physical_device,
      graphics_queue_family_index,
      present_queue_family_index,
      transfer_queue_family_index,
      vk_device,
      graphics_queue,
      present_queue,
      transfer_queue,
      vk_allocator,
      model_buffer,
      model_buffer_alloc,
      model_buffer_addr,
      model_buffer_data,
      swapchain_device,
      vk_surface_format,
      render_pass,
      graphics_command_pools,
      graphics_command_buffers,
      transition_command_pool,
      transition_command_buffer,
      transfer_command_pools,
      transfer_command_buffers,
      descriptor_pool,
      image_avail_semaphores,
      render_done_semaphores,
      in_flight_fences,
      transition_done_semaphore,
      glyph_atlas_semaphores,
      read_atlas_semaphores,
      pipeline_cache,
      msaa_samples,
      atlas_sizes,
      cam_position: None,
      cam_size: None,
      state: Creating {
        text_renderer,
        round_rect_renderer,
      },
    }
  }

  pub(crate) fn finish(self, window: Window) -> Result<Renderer<Created>, FinishError> {
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

    let (msaa_color_image, msaa_color_image_alloc, msaa_color_image_view) =
      if self.msaa_samples > vk::SampleCountFlags::TYPE_1 {
        let msaa_color_image_create_info = vk::ImageCreateInfo {
          image_type: vk::ImageType::TYPE_2D,
          format: self.vk_surface_format.format,
          extent: vk::Extent3D {
            width: swapchain_image_extent.width,
            height: swapchain_image_extent.height,
            depth: 1,
          },
          mip_levels: 1,
          array_layers: 1,
          samples: self.msaa_samples,
          tiling: vk::ImageTiling::OPTIMAL,
          usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
          sharing_mode: vk::SharingMode::EXCLUSIVE,
          initial_layout: vk::ImageLayout::UNDEFINED,
          ..Default::default()
        };

        let msaa_color_image_alloc_create_info = vk_mem::AllocationCreateInfo {
          flags: vk_mem::AllocationCreateFlags::DEDICATED_MEMORY,
          usage: vk_mem::MemoryUsage::AutoPreferDevice,
          preferred_flags: vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
          ..Default::default()
        };

        let (msaa_color_image, msaa_color_image_alloc) = unsafe {
          self
            .vk_allocator
            .create_image(
              &msaa_color_image_create_info,
              &msaa_color_image_alloc_create_info,
            )
            .unwrap()
        };

        let msaa_color_image_view_create_info = vk::ImageViewCreateInfo {
          image: msaa_color_image,
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

        let msaa_color_image_view = unsafe {
          self
            .vk_device
            .create_image_view(&msaa_color_image_view_create_info, None)
            .unwrap()
        };

        (
          msaa_color_image,
          Some(msaa_color_image_alloc),
          msaa_color_image_view,
        )
      } else {
        (vk::Image::null(), None, vk::ImageView::null())
      };

    let framebuffers = swapchain_image_views
      .iter()
      .map(|&swapchain_image_view| {
        let mut attachments = vec![];

        if !msaa_color_image_view.is_null() {
          attachments.push(msaa_color_image_view);
        }

        attachments.push(swapchain_image_view);

        let framebuffer_create_info = vk::FramebufferCreateInfo {
          render_pass: self.render_pass,
          attachment_count: attachments.len() as _,
          p_attachments: attachments.as_ptr(),
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

    let text_renderer = self.state.text_renderer.finish(
      &self.vk_device,
      self.render_pass,
      self.pipeline_cache,
      swapchain_image_extent,
      self.msaa_samples,
    );

    let round_rect_renderer = self.state.round_rect_renderer.finish(
      &self.vk_device,
      self.render_pass,
      self.pipeline_cache,
      swapchain_image_extent,
      self.msaa_samples,
    );

    Ok(Renderer {
      ash_entry: self.ash_entry,
      vk_inst: self.vk_inst,
      vk_surface: self.vk_surface,
      surface_inst: self.surface_inst,
      vk_physical_device: self.vk_physical_device,
      graphics_queue_family_index: self.graphics_queue_family_index,
      present_queue_family_index: self.present_queue_family_index,
      transfer_queue_family_index: self.transfer_queue_family_index,
      vk_device: self.vk_device,
      graphics_queue: self.graphics_queue,
      present_queue: self.present_queue,
      transfer_queue: self.transfer_queue,
      vk_allocator: self.vk_allocator,
      model_buffer: self.model_buffer,
      model_buffer_alloc: self.model_buffer_alloc,
      model_buffer_addr: self.model_buffer_addr,
      model_buffer_data: self.model_buffer_data,
      swapchain_device: self.swapchain_device,
      vk_surface_format: self.vk_surface_format,
      render_pass: self.render_pass,
      graphics_command_pools: self.graphics_command_pools,
      graphics_command_buffers: self.graphics_command_buffers,
      transition_command_pool: self.transition_command_pool,
      transition_command_buffer: self.transition_command_buffer,
      transfer_command_pools: self.transfer_command_pools,
      transfer_command_buffers: self.transfer_command_buffers,
      descriptor_pool: self.descriptor_pool,
      image_avail_semaphores: self.image_avail_semaphores,
      render_done_semaphores: self.render_done_semaphores,
      in_flight_fences: self.in_flight_fences,
      transition_done_semaphore: self.transition_done_semaphore,
      glyph_atlas_semaphores: self.glyph_atlas_semaphores,
      read_atlas_semaphores: self.read_atlas_semaphores,
      pipeline_cache: self.pipeline_cache,
      msaa_samples: self.msaa_samples,
      atlas_sizes: self.atlas_sizes,
      cam_position: self.cam_position,
      cam_size: self.cam_size,
      state: Created {
        swapchain_image_extent,
        swapchain,
        _swapchain_images: swapchain_images,
        swapchain_image_views,
        msaa_color_image,
        msaa_color_image_alloc,
        msaa_color_image_view,
        framebuffers,
        text_renderer,
        round_rect_renderer,
        frame_index: 0,
      },
    })
  }

  #[inline]
  pub(super) const fn get_text_renderer(&mut self) -> &mut TextRenderer<text_renderer::Creating> {
    &mut self.state.text_renderer
  }

  #[inline]
  pub(super) const fn get_round_rect_renderer(
    &mut self,
  ) -> &mut ModelRenderer<model_renderer::Creating<RoundRect>> {
    &mut self.state.round_rect_renderer
  }

  pub(crate) fn drop(mut self) {
    unsafe {
      self.vk_device.device_wait_idle().unwrap();
      self.state.round_rect_renderer.drop(&self.vk_device);

      self
        .state
        .text_renderer
        .drop(&self.vk_device, &self.vk_allocator);

      self
        .vk_device
        .destroy_pipeline_cache(self.pipeline_cache, None);

      self
        .read_atlas_semaphores
        .iter()
        .for_each(|&(semaphore, _)| self.vk_device.destroy_semaphore(semaphore, None));

      self
        .glyph_atlas_semaphores
        .iter()
        .for_each(|&(semaphore, _)| self.vk_device.destroy_semaphore(semaphore, None));

      self
        .vk_device
        .destroy_semaphore(self.transition_done_semaphore, None);

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
        .vk_device
        .destroy_descriptor_pool(self.descriptor_pool, None);

      self
        .transfer_command_pools
        .iter()
        .for_each(|&command_pool| self.vk_device.destroy_command_pool(command_pool, None));

      self
        .vk_device
        .destroy_command_pool(self.transition_command_pool, None);

      self
        .graphics_command_pools
        .iter()
        .for_each(|&command_pool| self.vk_device.destroy_command_pool(command_pool, None));

      self.vk_device.destroy_render_pass(self.render_pass, None);

      self
        .vk_allocator
        .destroy_buffer(self.model_buffer, &mut self.model_buffer_alloc);

      drop(self.vk_allocator);
      self.vk_device.destroy_device(None);
      self.surface_inst.destroy_surface(self.vk_surface, None);
      self.vk_inst.destroy_instance(None);
    }
  }
}

impl Renderer<Created> {
  #[inline]
  pub(super) const fn get_text_renderer(&mut self) -> &mut TextRenderer<text_renderer::Created> {
    &mut self.state.text_renderer
  }

  #[inline]
  pub(super) const fn get_round_rect_renderer(
    &mut self,
  ) -> &mut ModelRenderer<model_renderer::Created<RoundRect>> {
    &mut self.state.round_rect_renderer
  }

  pub(crate) fn render(mut self, window: Window) -> AnyRenderer {
    let graphics_command_pool = self.graphics_command_pools[self.state.frame_index];
    let graphics_command_buffer = self.graphics_command_buffers[self.state.frame_index];
    let image_avail_semaphore = self.image_avail_semaphores[self.state.frame_index];
    let render_done_semaphore = self.render_done_semaphores[self.state.frame_index];
    let in_flight_fence = self.in_flight_fences[self.state.frame_index];
    let write_image_index = self.state.text_renderer.get_write_image_index();
    let transfer_command_pool = self.transfer_command_pools[write_image_index];
    let transfer_command_buffer = self.transfer_command_buffers[write_image_index];
    let (glyph_atlas_semaphore, glyph_atlas_semaphore_value) =
      self.glyph_atlas_semaphores[write_image_index];
    let (read_atlas_semaphore, read_atlas_semaphore_value) =
      self.read_atlas_semaphores[write_image_index];

    let glyph_atlas_semaphore_value = if self.state.text_renderer.flush_atlas_updates(
      &self.vk_device,
      glyph_atlas_semaphore,
      glyph_atlas_semaphore_value,
    ) {
      unsafe {
        self
          .vk_device
          .reset_command_pool(transfer_command_pool, vk::CommandPoolResetFlags::empty())
          .unwrap();

        let transfer_command_buffer_begin_info = vk::CommandBufferBeginInfo {
          flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
          ..Default::default()
        };

        self
          .vk_device
          .begin_command_buffer(transfer_command_buffer, &transfer_command_buffer_begin_info)
          .unwrap();

        self.state.text_renderer.record_copy_commands(
          &self.vk_device,
          transfer_command_buffer,
          self.graphics_queue_family_index,
          self.transfer_queue_family_index,
        );

        self
          .vk_device
          .end_command_buffer(transfer_command_buffer)
          .unwrap();

        let (extra_transfer_wait_semaphore, extra_transfer_wait_semaphore_value) =
          if read_atlas_semaphore_value > 0 {
            (read_atlas_semaphore, read_atlas_semaphore_value)
          } else {
            (self.transition_done_semaphore, 1)
          };

        let wait_semaphores = [extra_transfer_wait_semaphore, glyph_atlas_semaphore];
        let wait_semaphore_values = [
          extra_transfer_wait_semaphore_value,
          glyph_atlas_semaphore_value + 1,
        ];
        let signal_semaphore = glyph_atlas_semaphore;
        let signal_semaphore_value = glyph_atlas_semaphore_value + 2;

        let transfer_semaphore_submit_info = vk::TimelineSemaphoreSubmitInfo {
          wait_semaphore_value_count: wait_semaphore_values.len() as _,
          p_wait_semaphore_values: wait_semaphore_values.as_ptr(),
          signal_semaphore_value_count: 1,
          p_signal_semaphore_values: &signal_semaphore_value,
          ..Default::default()
        };

        let wait_dst_stage_masks = [
          vk::PipelineStageFlags::TRANSFER,
          vk::PipelineStageFlags::TRANSFER,
        ];

        let transfer_queue_submit_info = vk::SubmitInfo {
          wait_semaphore_count: wait_semaphores.len() as _,
          p_wait_semaphores: wait_semaphores.as_ptr(),
          p_wait_dst_stage_mask: wait_dst_stage_masks.as_ptr(),
          command_buffer_count: 1,
          p_command_buffers: &transfer_command_buffer,
          signal_semaphore_count: 1,
          p_signal_semaphores: &signal_semaphore,
          p_next: &transfer_semaphore_submit_info as *const _ as *const _,
          ..Default::default()
        };

        self
          .vk_device
          .queue_submit(
            self.transfer_queue,
            &[transfer_queue_submit_info],
            vk::Fence::null(),
          )
          .unwrap();
      }

      glyph_atlas_semaphore_value + 2
    } else {
      glyph_atlas_semaphore_value
    };

    let read_image_index = self.state.text_renderer.get_read_image_index();
    let (read_atlas_semaphore, read_atlas_semaphore_value) =
      self.read_atlas_semaphores[read_image_index];

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

      let glyph_capacity_size = self
        .state
        .text_renderer
        .get_glyph_renderer()
        .get_model_capacity()
        * mem::size_of::<Glyph>();

      let round_rect_capacity_size =
        self.state.round_rect_renderer.get_model_capacity() * mem::size_of::<RoundRect>();

      let total_capacity_size = glyph_capacity_size + round_rect_capacity_size;

      self
        .state
        .text_renderer
        .get_glyph_renderer_mut()
        .flush_writes(
          self
            .model_buffer_data
            .byte_add(self.state.frame_index * total_capacity_size) as *mut Glyph,
        );

      self.state.round_rect_renderer.flush_writes(
        self
          .model_buffer_data
          .byte_add(self.state.frame_index * total_capacity_size + glyph_capacity_size)
          as *mut RoundRect,
      );

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

      let cam_position = self.cam_position.unwrap_or_default();

      let cam_size = self.cam_size.unwrap_or((
        self.state.swapchain_image_extent.width as _,
        self.state.swapchain_image_extent.height as _,
      ));

      let window_display_scale = window.display_scale();

      let actual_cam_position = (
        cam_position.0 / window_display_scale,
        cam_position.1 / window_display_scale,
      );

      let actual_cam_size = (
        cam_size.0 / window_display_scale,
        cam_size.1 / window_display_scale,
      );

      let glyph_push_consts = GlyphPushConsts {
        glyph_buffer: self.model_buffer_addr
          + (self.state.frame_index * total_capacity_size) as u64,
        cam_position: actual_cam_position,
        cam_size: actual_cam_size,
        atlas_size: (
          self.atlas_sizes.glyph_atlas_size.0 as _,
          self.atlas_sizes.glyph_atlas_size.1 as _,
        ),
      };

      let round_rect_push_consts = RoundRectPushConsts {
        round_rect_buffer: self.model_buffer_addr
          + (self.state.frame_index * total_capacity_size + glyph_capacity_size) as u64,
        cam_position: actual_cam_position,
        cam_size: actual_cam_size,
      };

      self
        .state
        .text_renderer
        .get_glyph_renderer()
        .record_draw_commands(
          &self.vk_device,
          graphics_command_buffer,
          self.state.text_renderer.get_descriptor_set(),
          &glyph_push_consts,
        );

      self.state.round_rect_renderer.record_draw_commands(
        &self.vk_device,
        graphics_command_buffer,
        vk::DescriptorSet::null(),
        &round_rect_push_consts,
      );

      let subpass_end_info = vk::SubpassEndInfo::default();

      self
        .vk_device
        .cmd_end_render_pass2(graphics_command_buffer, &subpass_end_info);

      self
        .vk_device
        .end_command_buffer(graphics_command_buffer)
        .unwrap();

      self.vk_device.reset_fences(&[in_flight_fence]).unwrap();

      let wait_semaphores = [image_avail_semaphore, glyph_atlas_semaphore];
      let wait_semaphore_values = [0, glyph_atlas_semaphore_value];
      let signal_semaphores = [render_done_semaphore, read_atlas_semaphore];
      let signal_semaphore_values = [0, read_atlas_semaphore_value + 1];

      let graphics_semaphore_submit_info = vk::TimelineSemaphoreSubmitInfo {
        wait_semaphore_value_count: wait_semaphore_values.len() as _,
        p_wait_semaphore_values: wait_semaphore_values.as_ptr(),
        signal_semaphore_value_count: signal_semaphore_values.len() as _,
        p_signal_semaphore_values: signal_semaphore_values.as_ptr(),
        ..Default::default()
      };

      let wait_dst_stage_masks = [
        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
      ];

      let graphics_queue_submit_info = vk::SubmitInfo {
        wait_semaphore_count: wait_semaphores.len() as _,
        p_wait_semaphores: wait_semaphores.as_ptr(),
        p_wait_dst_stage_mask: wait_dst_stage_masks.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &graphics_command_buffer,
        signal_semaphore_count: signal_semaphores.len() as _,
        p_signal_semaphores: signal_semaphores.as_ptr(),
        p_next: &graphics_semaphore_submit_info as *const _ as *const _,
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

    self.glyph_atlas_semaphores[write_image_index].1 = glyph_atlas_semaphore_value;
    self.read_atlas_semaphores[read_image_index].1 = read_atlas_semaphore_value + 1;

    AnyRenderer::Created(Self {
      state: Created {
        frame_index: (self.state.frame_index + 1) % MAX_IN_FLIGHT_FRAME_COUNT,
        ..self.state
      },
      ..self
    })
  }

  pub(crate) fn on_swapchain_suboptimal(self) -> Renderer<Creating> {
    unsafe {
      self.vk_device.device_wait_idle().unwrap();
    }

    let text_renderer = self.state.text_renderer.on_swapchain_suboptimal();
    let round_rect_renderer = self.state.round_rect_renderer.on_swapchain_suboptimal();

    unsafe {
      self
        .state
        .framebuffers
        .iter()
        .for_each(|&framebuffer| self.vk_device.destroy_framebuffer(framebuffer, None));

      self
        .vk_device
        .destroy_image_view(self.state.msaa_color_image_view, None);

      if let Some(mut msaa_color_image_alloc) = self.state.msaa_color_image_alloc {
        self
          .vk_allocator
          .destroy_image(self.state.msaa_color_image, &mut msaa_color_image_alloc);
      }

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
      transfer_queue_family_index: self.transfer_queue_family_index,
      vk_device: self.vk_device,
      graphics_queue: self.graphics_queue,
      present_queue: self.present_queue,
      transfer_queue: self.transfer_queue,
      vk_allocator: self.vk_allocator,
      model_buffer: self.model_buffer,
      model_buffer_alloc: self.model_buffer_alloc,
      model_buffer_addr: self.model_buffer_addr,
      model_buffer_data: self.model_buffer_data,
      swapchain_device: self.swapchain_device,
      vk_surface_format: self.vk_surface_format,
      render_pass: self.render_pass,
      graphics_command_pools: self.graphics_command_pools,
      graphics_command_buffers: self.graphics_command_buffers,
      transition_command_pool: self.transition_command_pool,
      transition_command_buffer: self.transition_command_buffer,
      transfer_command_pools: self.transfer_command_pools,
      transfer_command_buffers: self.transfer_command_buffers,
      descriptor_pool: self.descriptor_pool,
      image_avail_semaphores: self.image_avail_semaphores,
      render_done_semaphores: self.render_done_semaphores,
      in_flight_fences: self.in_flight_fences,
      transition_done_semaphore: self.transition_done_semaphore,
      glyph_atlas_semaphores: self.glyph_atlas_semaphores,
      read_atlas_semaphores: self.read_atlas_semaphores,
      pipeline_cache: self.pipeline_cache,
      msaa_samples: self.msaa_samples,
      atlas_sizes: self.atlas_sizes,
      cam_position: self.cam_position,
      cam_size: self.cam_size,
      state: Creating {
        text_renderer,
        round_rect_renderer,
      },
    }
  }

  pub(crate) fn drop(mut self) {
    unsafe {
      self.vk_device.device_wait_idle().unwrap();
      self.state.round_rect_renderer.drop(&self.vk_device);

      self
        .state
        .text_renderer
        .drop(&self.vk_device, &self.vk_allocator);

      self
        .state
        .framebuffers
        .iter()
        .for_each(|&framebuffer| self.vk_device.destroy_framebuffer(framebuffer, None));

      self
        .vk_device
        .destroy_image_view(self.state.msaa_color_image_view, None);

      if let Some(mut msaa_color_image_alloc) = self.state.msaa_color_image_alloc {
        self
          .vk_allocator
          .destroy_image(self.state.msaa_color_image, &mut msaa_color_image_alloc);
      }

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
        .read_atlas_semaphores
        .iter()
        .for_each(|&(semaphore, _)| self.vk_device.destroy_semaphore(semaphore, None));

      self
        .glyph_atlas_semaphores
        .iter()
        .for_each(|&(semaphore, _)| self.vk_device.destroy_semaphore(semaphore, None));

      self
        .vk_device
        .destroy_semaphore(self.transition_done_semaphore, None);

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
        .vk_device
        .destroy_descriptor_pool(self.descriptor_pool, None);

      self
        .transfer_command_pools
        .iter()
        .for_each(|&command_pool| self.vk_device.destroy_command_pool(command_pool, None));

      self
        .vk_device
        .destroy_command_pool(self.transition_command_pool, None);

      self
        .graphics_command_pools
        .iter()
        .for_each(|&command_pool| self.vk_device.destroy_command_pool(command_pool, None));

      self.vk_device.destroy_render_pass(self.render_pass, None);

      self
        .vk_allocator
        .destroy_buffer(self.model_buffer, &mut self.model_buffer_alloc);

      drop(self.vk_allocator);
      self.vk_device.destroy_device(None);
      self.surface_inst.destroy_surface(self.vk_surface, None);
      self.vk_inst.destroy_instance(None);
    }
  }
}

impl<State> Renderer<State> {
  #[inline]
  pub(super) fn set_cam_position(&mut self, cam_position: Option<(f32, f32)>) {
    self.cam_position = cam_position;
  }

  #[inline]
  pub(super) fn set_cam_size(&mut self, cam_size: Option<(f32, f32)>) {
    self.cam_size = cam_size;
  }
}
