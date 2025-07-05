use super::{Device, GraphicsPipeline, graphics_pipeline};
use ash::vk::{self, Handle};
use sdl2::video::Window;
use std::{ffi::CString, iter, rc::Rc};

// Settings
const SWAPCHAIN_IMAGE_FORMAT: vk::Format = vk::Format::B8G8R8A8_UNORM;
const MAX_IN_FLIGHT_FRAME_COUNT: usize = 3;

pub(crate) struct Creating {
  rect_pipeline: GraphicsPipeline<graphics_pipeline::Creating>,
}

pub(crate) struct Created {
  swapchain_image_extent: vk::Extent2D,
  swapchain: vk::SwapchainKHR,
  swapchain_image_views: Box<[vk::ImageView]>,
  swapchain_framebuffers: Box<[vk::Framebuffer]>,
  rect_pipeline: GraphicsPipeline<graphics_pipeline::Created>,
  frame_index: usize,
}

pub(crate) struct Renderer<State> {
  window: Window,
  entry: ash::Entry,
  instance: ash::Instance,
  surface_instance: ash::khr::surface::Instance,
  surface: vk::SurfaceKHR,
  physical_device: vk::PhysicalDevice,
  graphics_queue_family_index: u32,
  present_queue_family_index: u32,
  device: Rc<Device>,
  graphics_queue: vk::Queue,
  present_queue: vk::Queue,
  graphics_command_pools: Box<[vk::CommandPool]>,
  graphics_command_buffers: Box<[vk::CommandBuffer]>,
  swapchain_device: ash::khr::swapchain::Device,
  render_pass: vk::RenderPass,
  image_avail_semaphores: Box<[vk::Semaphore]>,
  render_finished_semaphores: Box<[vk::Semaphore]>,
  in_flight_fences: Box<[vk::Fence]>,
  state: State,
}

impl Renderer<Creating> {
  pub(crate) fn new(window: Window) -> Self {
    let entry = unsafe { ash::Entry::load().unwrap() };
    let app_name = CString::new(window.title()).unwrap();

    let app_info = vk::ApplicationInfo {
      p_application_name: app_name.as_ptr(),
      application_version: vk::make_api_version(0, 0, 1, 0),
      api_version: vk::make_api_version(0, 1, 3, 0),
      ..Default::default()
    };

    let instance_ext_names = window
      .vulkan_instance_extensions()
      .unwrap()
      .into_iter()
      .map(|ext_name| ext_name.as_ptr() as *const _)
      .chain(iter::once(vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr()))
      .collect::<Box<_>>();

    #[cfg(debug_assertions)]
    let instance = {
      let vk_layer_khronos_validation_name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
      let validate_sync_name = CString::new("validate_sync").unwrap();
      let gpuav_enable_name = CString::new("gpuav_enable").unwrap();
      let validate_best_practices_name = CString::new("validate_best_practices").unwrap();
      let validate_best_practices_arm_name = CString::new("validate_best_practices_arm").unwrap();
      let validate_best_practices_amd_name = CString::new("validate_best_practices_amd").unwrap();
      let validate_best_practices_img_name = CString::new("validate_best_practices_img").unwrap();
      let validate_best_practices_nvidia_name =
        CString::new("validate_best_practices_nvidia").unwrap();
      let report_flags_name = CString::new("report_flags").unwrap();
      let gpuav_validate_ray_query_name = CString::new("gpuav_validate_ray_query").unwrap();
      let warn_name = CString::new("warn").unwrap();
      let perf_name = CString::new("perf").unwrap();
      let error_name = CString::new("error").unwrap();
      let vk_true = vk::TRUE;
      let vk_false = vk::FALSE;
      let report_flags_values = [warn_name.as_ptr(), perf_name.as_ptr(), error_name.as_ptr()];
      let layer_names = [vk_layer_khronos_validation_name.as_ptr()];

      let layer_settings = [
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: validate_sync_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::BOOL32,
          value_count: 1,
          p_values: &vk_true as *const _ as *const _,
          ..Default::default()
        },
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: gpuav_enable_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::BOOL32,
          value_count: 1,
          p_values: &vk_true as *const _ as *const _,
          ..Default::default()
        },
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: validate_best_practices_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::BOOL32,
          value_count: 1,
          p_values: &vk_true as *const _ as *const _,
          ..Default::default()
        },
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: validate_best_practices_arm_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::BOOL32,
          value_count: 1,
          p_values: &vk_true as *const _ as *const _,
          ..Default::default()
        },
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: validate_best_practices_amd_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::BOOL32,
          value_count: 1,
          p_values: &vk_true as *const _ as *const _,
          ..Default::default()
        },
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: validate_best_practices_img_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::BOOL32,
          value_count: 1,
          p_values: &vk_true as *const _ as *const _,
          ..Default::default()
        },
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: validate_best_practices_nvidia_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::BOOL32,
          value_count: 1,
          p_values: &vk_true as *const _ as *const _,
          ..Default::default()
        },
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: report_flags_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::STRING,
          value_count: report_flags_values.len() as _,
          p_values: report_flags_values.as_ptr() as *const _,
          ..Default::default()
        },
        vk::LayerSettingEXT {
          p_layer_name: vk_layer_khronos_validation_name.as_ptr(),
          p_setting_name: gpuav_validate_ray_query_name.as_ptr(),
          ty: vk::LayerSettingTypeEXT::BOOL32,
          value_count: 1,
          p_values: &vk_false as *const _ as *const _,
          ..Default::default()
        },
      ];

      let layer_settings_create_info = vk::LayerSettingsCreateInfoEXT {
        setting_count: layer_settings.len() as _,
        p_settings: layer_settings.as_ptr(),
        ..Default::default()
      };

      let instance_create_info = vk::InstanceCreateInfo {
        flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
        p_application_info: &app_info,
        enabled_layer_count: layer_names.len() as _,
        pp_enabled_layer_names: layer_names.as_ptr(),
        enabled_extension_count: instance_ext_names.len() as _,
        pp_enabled_extension_names: instance_ext_names.as_ptr(),
        p_next: &layer_settings_create_info as *const _ as *const _,
        ..Default::default()
      };

      unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    };

    #[cfg(not(debug_assertions))]
    let instance = {
      let instance_create_info = vk::InstanceCreateInfo {
        flags: vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
        p_application_info: &app_info,
        enabled_extension_count: instance_ext_names.len() as _,
        pp_enabled_extension_names: instance_ext_names.as_ptr(),
        ..Default::default()
      };

      unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    };

    let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);

    let surface = vk::SurfaceKHR::from_raw(
      window
        .vulkan_create_surface(instance.handle().as_raw() as _)
        .unwrap(),
    );

    let (graphics_queue_family_index, present_queue_family_index, physical_device) = unsafe {
      instance
        .enumerate_physical_devices()
        .unwrap()
        .into_iter()
        .filter_map(|physical_device| {
          let mut queue_family_props_list =
            vec![
              vk::QueueFamilyProperties2::default();
              instance.get_physical_device_queue_family_properties2_len(physical_device)
            ];

          instance.get_physical_device_queue_family_properties2(
            physical_device,
            queue_family_props_list.as_mut_slice(),
          );

          if let Some((queue_family_index, _)) = queue_family_props_list
            .iter()
            .enumerate()
            .filter(|&(index, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .contains(vk::QueueFlags::GRAPHICS)
                && surface_instance
                  .get_physical_device_surface_support(physical_device, index as _, surface)
                  .unwrap()
            })
            .min_by_key(|&(_, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .as_raw()
                .count_ones()
            })
          {
            return Some((
              queue_family_index as _,
              queue_family_index as _,
              physical_device,
            ));
          }

          let (graphics_queue_family_index, _) = queue_family_props_list
            .iter()
            .enumerate()
            .filter(|&(_, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .contains(vk::QueueFlags::GRAPHICS)
            })
            .min_by_key(|&(_, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .as_raw()
                .count_ones()
            })?;

          let (present_queue_family_index, _) = queue_family_props_list
            .iter()
            .enumerate()
            .filter(|&(index, _)| {
              surface_instance
                .get_physical_device_surface_support(physical_device, index as _, surface)
                .unwrap()
            })
            .min_by_key(|&(_, queue_family_props)| {
              queue_family_props
                .queue_family_properties
                .queue_flags
                .as_raw()
                .count_ones()
            })?;

          Some((
            graphics_queue_family_index as _,
            present_queue_family_index as _,
            physical_device,
          ))
        })
        .max_by_key(|&(_, _, physical_device)| {
          let mut physical_device_props = vk::PhysicalDeviceProperties2::default();
          instance.get_physical_device_properties2(physical_device, &mut physical_device_props);

          match physical_device_props.properties.device_type {
            vk::PhysicalDeviceType::INTEGRATED_GPU => 4,
            vk::PhysicalDeviceType::DISCRETE_GPU => 3,
            vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
            vk::PhysicalDeviceType::CPU => 1,
            _ => 0,
          }
        })
        .unwrap()
    };

    let vk_device = Rc::new(Device::new(
      &instance,
      physical_device,
      graphics_queue_family_index,
      present_queue_family_index,
    ));

    let device = vk_device.get();

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

    let graphics_queue = unsafe { device.get_device_queue2(&graphics_queue_info) };
    let present_queue = unsafe { device.get_device_queue2(&present_queue_info) };

    let graphics_command_pool_create_info = vk::CommandPoolCreateInfo {
      flags: vk::CommandPoolCreateFlags::TRANSIENT,
      queue_family_index: graphics_queue_family_index,
      ..Default::default()
    };

    let graphics_command_pools = (0..MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|_| unsafe {
        device
          .create_command_pool(&graphics_command_pool_create_info, None)
          .unwrap()
      })
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
          device
            .allocate_command_buffers(&command_buffer_alloc_info)
            .unwrap()[0]
        }
      })
      .collect::<Box<_>>();

    let rect_pipeline = GraphicsPipeline::new(
      vk_device.clone(),
      include_bytes!("../../target/spv/rect.vert.spv"),
      include_bytes!("../../target/spv/rect.frag.spv"),
    );

    let swapchain_device = ash::khr::swapchain::Device::new(&instance, device);

    let attachment_descs = [vk::AttachmentDescription2 {
      format: SWAPCHAIN_IMAGE_FORMAT,
      samples: vk::SampleCountFlags::TYPE_1,
      load_op: vk::AttachmentLoadOp::CLEAR,
      store_op: vk::AttachmentStoreOp::STORE,
      stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
      stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
      initial_layout: vk::ImageLayout::UNDEFINED,
      final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
      ..Default::default()
    }];

    let color_attachemnt_refs = [vk::AttachmentReference2 {
      attachment: 0,
      layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
      ..Default::default()
    }];

    let subpass_descs = [vk::SubpassDescription2 {
      pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
      color_attachment_count: color_attachemnt_refs.len() as _,
      p_color_attachments: color_attachemnt_refs.as_ptr(),
      ..Default::default()
    }];

    let subpass_deps = [
      vk::SubpassDependency2 {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: vk::AccessFlags::NONE,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::BY_REGION,
        ..Default::default()
      },
      vk::SubpassDependency2 {
        src_subpass: 0,
        dst_subpass: vk::SUBPASS_EXTERNAL,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dst_access_mask: vk::AccessFlags::NONE,
        dependency_flags: vk::DependencyFlags::BY_REGION,
        ..Default::default()
      },
    ];

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
      device
        .create_render_pass2(&render_pass_create_info, None)
        .unwrap()
    };

    let (image_avail_semaphores, (render_finished_semaphores, in_flight_fences)): (
      Vec<_>,
      (Vec<_>, Vec<_>),
    ) = (0..MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|_| {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let fence_create_info = vk::FenceCreateInfo {
          flags: vk::FenceCreateFlags::SIGNALED,
          ..Default::default()
        };

        unsafe {
          (
            device
              .create_semaphore(&semaphore_create_info, None)
              .unwrap(),
            (
              device
                .create_semaphore(&semaphore_create_info, None)
                .unwrap(),
              device.create_fence(&fence_create_info, None).unwrap(),
            ),
          )
        }
      })
      .unzip();

    Self {
      window,
      entry,
      instance,
      surface_instance,
      surface,
      physical_device,
      graphics_queue_family_index,
      present_queue_family_index,
      device: vk_device,
      graphics_queue,
      present_queue,
      graphics_command_pools,
      graphics_command_buffers,
      swapchain_device,
      render_pass,
      image_avail_semaphores: image_avail_semaphores.into_boxed_slice(),
      render_finished_semaphores: render_finished_semaphores.into_boxed_slice(),
      in_flight_fences: in_flight_fences.into_boxed_slice(),
      state: Creating { rect_pipeline },
    }
  }

  #[allow(clippy::result_large_err)]
  pub(crate) fn finish(self) -> Result<Renderer<Created>, Self> {
    let surface_caps = unsafe {
      self
        .surface_instance
        .get_physical_device_surface_capabilities(self.physical_device, self.surface)
        .unwrap()
    };

    let swapchain_min_image_count = surface_caps.min_image_count + 1;

    let swapchain_min_image_count = if surface_caps.max_image_count > 0 {
      swapchain_min_image_count.min(surface_caps.max_image_count)
    } else {
      swapchain_min_image_count
    };

    let swapchain_image_extent = if surface_caps.current_extent.width != u32::MAX {
      surface_caps.current_extent
    } else {
      let drawable_size = self.window.vulkan_drawable_size();

      vk::Extent2D {
        width: drawable_size.0.clamp(
          surface_caps.min_image_extent.width,
          surface_caps.max_image_extent.width,
        ),
        height: drawable_size.1.clamp(
          surface_caps.min_image_extent.height,
          surface_caps.max_image_extent.height,
        ),
      }
    };

    if swapchain_image_extent.width == 0 || swapchain_image_extent.height == 0 {
      return Err(self);
    }

    let queue_family_indices = [
      self.graphics_queue_family_index,
      self.present_queue_family_index,
    ];

    let (swapchain_image_sharing_mode, queue_family_indices) =
      if self.graphics_queue_family_index == self.present_queue_family_index {
        (vk::SharingMode::EXCLUSIVE, [].as_slice())
      } else {
        (vk::SharingMode::CONCURRENT, queue_family_indices.as_slice())
      };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
      surface: self.surface,
      min_image_count: swapchain_min_image_count,
      image_format: SWAPCHAIN_IMAGE_FORMAT,
      image_extent: swapchain_image_extent,
      image_array_layers: 1,
      image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
      image_sharing_mode: swapchain_image_sharing_mode,
      queue_family_index_count: queue_family_indices.len() as _,
      p_queue_family_indices: queue_family_indices.as_ptr(),
      pre_transform: surface_caps.current_transform,
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

    let device = self.device.get();

    let swapchain_image_views = swapchain_images
      .iter()
      .map(|&image| {
        let image_view_create_info = vk::ImageViewCreateInfo {
          image,
          view_type: vk::ImageViewType::TYPE_2D,
          format: SWAPCHAIN_IMAGE_FORMAT,
          components: vk::ComponentMapping::default(),
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
          device
            .create_image_view(&image_view_create_info, None)
            .unwrap()
        }
      })
      .collect::<Box<_>>();

    let swapchain_framebuffers = swapchain_image_views
      .iter()
      .map(|&image_view| {
        let swapchain_framebuffer_create_info = vk::FramebufferCreateInfo {
          render_pass: self.render_pass,
          attachment_count: 1,
          p_attachments: &image_view,
          width: swapchain_image_extent.width,
          height: swapchain_image_extent.height,
          layers: 1,
          ..Default::default()
        };

        unsafe {
          device
            .create_framebuffer(&swapchain_framebuffer_create_info, None)
            .unwrap()
        }
      })
      .collect::<Box<_>>();

    let rect_pipeline = self
      .state
      .rect_pipeline
      .finish(self.render_pass, swapchain_image_extent);

    Ok(Renderer {
      window: self.window,
      entry: self.entry,
      instance: self.instance,
      surface_instance: self.surface_instance,
      surface: self.surface,
      physical_device: self.physical_device,
      graphics_queue_family_index: self.graphics_queue_family_index,
      present_queue_family_index: self.present_queue_family_index,
      device: self.device,
      graphics_queue: self.graphics_queue,
      present_queue: self.present_queue,
      graphics_command_pools: self.graphics_command_pools,
      graphics_command_buffers: self.graphics_command_buffers,
      swapchain_device: self.swapchain_device,
      render_pass: self.render_pass,
      image_avail_semaphores: self.image_avail_semaphores,
      render_finished_semaphores: self.render_finished_semaphores,
      in_flight_fences: self.in_flight_fences,
      state: Created {
        swapchain_image_extent,
        swapchain,
        swapchain_image_views,
        swapchain_framebuffers,
        rect_pipeline,
        frame_index: 0,
      },
    })
  }

  pub(crate) fn drop(self) {
    let device = self.device.get();

    unsafe {
      device.device_wait_idle().unwrap();

      self
        .in_flight_fences
        .into_iter()
        .for_each(|fence| device.destroy_fence(fence, None));

      self
        .render_finished_semaphores
        .into_iter()
        .for_each(|semaphore| device.destroy_semaphore(semaphore, None));

      self
        .image_avail_semaphores
        .into_iter()
        .for_each(|semaphore| device.destroy_semaphore(semaphore, None));

      device.destroy_render_pass(self.render_pass, None);

      self
        .graphics_command_pools
        .into_iter()
        .for_each(|command_pool| device.destroy_command_pool(command_pool, None));

      device.destroy_device(None);
      self.surface_instance.destroy_surface(self.surface, None);
      self.instance.destroy_instance(None);
    }
  }
}

impl Renderer<Created> {
  #[allow(clippy::result_large_err)]
  pub(crate) fn render(self) -> Result<Self, Renderer<Creating>> {
    let image_avail_semaphore = self.image_avail_semaphores[self.state.frame_index];
    let render_finished_semaphore = self.render_finished_semaphores[self.state.frame_index];
    let in_flight_fence = self.in_flight_fences[self.state.frame_index];
    let graphics_command_pool = self.graphics_command_pools[self.state.frame_index];
    let graphics_command_buffer = self.graphics_command_buffers[self.state.frame_index];
    let device = self.device.get();

    unsafe {
      device
        .wait_for_fences(&[in_flight_fence], true, u64::MAX)
        .unwrap();
    }

    let acquire_next_image_info = vk::AcquireNextImageInfoKHR {
      swapchain: self.state.swapchain,
      timeout: u64::MAX,
      semaphore: image_avail_semaphore,
      fence: vk::Fence::null(),
      device_mask: 1,
      ..Default::default()
    };

    let swapchain_image_index = unsafe {
      match self
        .swapchain_device
        .acquire_next_image2(&acquire_next_image_info)
      {
        Ok((swapchain_image_index, _swapchain_suboptimal)) => swapchain_image_index,
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return self.on_swapchain_suboptimal().finish(),
        Err(err) => panic!("Unexpected error: {err}"),
      }
    };

    let graphics_command_buffer_begin_info = vk::CommandBufferBeginInfo {
      flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    let swapchain_framebuffer = self.state.swapchain_framebuffers[swapchain_image_index as usize];

    let clear_values = [vk::ClearValue {
      color: vk::ClearColorValue {
        float32: [0.0, 0.0, 0.0, 1.0],
      },
    }];

    let render_pass_begin_info = vk::RenderPassBeginInfo {
      render_pass: self.render_pass,
      framebuffer: swapchain_framebuffer,
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

    let subpass_end_info = vk::SubpassEndInfo::default();

    unsafe {
      device
        .reset_command_pool(graphics_command_pool, vk::CommandPoolResetFlags::empty())
        .unwrap();

      device
        .begin_command_buffer(graphics_command_buffer, &graphics_command_buffer_begin_info)
        .unwrap();

      device.cmd_begin_render_pass2(
        graphics_command_buffer,
        &render_pass_begin_info,
        &subpass_begin_info,
      );

      device.cmd_bind_pipeline(
        graphics_command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.state.rect_pipeline.get(),
      );

      device.cmd_draw(graphics_command_buffer, 6, 1, 0, 0);
      device.cmd_end_render_pass2(graphics_command_buffer, &subpass_end_info);

      device.end_command_buffer(graphics_command_buffer).unwrap();

      device.reset_fences(&[in_flight_fence]).unwrap();
    }

    let wait_dst_stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;

    let queue_submit_infos = [vk::SubmitInfo {
      wait_semaphore_count: 1,
      p_wait_semaphores: &image_avail_semaphore,
      p_wait_dst_stage_mask: &wait_dst_stage_mask,
      command_buffer_count: 1,
      p_command_buffers: &graphics_command_buffer,
      signal_semaphore_count: 1,
      p_signal_semaphores: &render_finished_semaphore,
      ..Default::default()
    }];

    unsafe {
      device
        .queue_submit(self.graphics_queue, &queue_submit_infos, in_flight_fence)
        .unwrap();
    }

    let present_info = vk::PresentInfoKHR {
      wait_semaphore_count: 1,
      p_wait_semaphores: &render_finished_semaphore,
      swapchain_count: 1,
      p_swapchains: &self.state.swapchain,
      p_image_indices: &swapchain_image_index,
      ..Default::default()
    };

    unsafe {
      match self
        .swapchain_device
        .queue_present(self.present_queue, &present_info)
      {
        Ok(false) => {}
        Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
          return self.on_swapchain_suboptimal().finish();
        }
        Err(err) => panic!("Unexpected error: {err}"),
      }
    }

    let frame_index = (self.state.frame_index + 1) % MAX_IN_FLIGHT_FRAME_COUNT;

    Ok(Self {
      state: Created {
        frame_index,
        ..self.state
      },
      ..self
    })
  }

  fn on_swapchain_suboptimal(self) -> Renderer<Creating> {
    let rect_pipeline = self.state.rect_pipeline.on_swapchain_suboptimal();
    let device = self.device.get();

    unsafe {
      device.device_wait_idle().unwrap();

      self
        .state
        .swapchain_framebuffers
        .into_iter()
        .for_each(|framebuffer| device.destroy_framebuffer(framebuffer, None));

      self
        .state
        .swapchain_image_views
        .into_iter()
        .for_each(|image_view| device.destroy_image_view(image_view, None));

      self
        .swapchain_device
        .destroy_swapchain(self.state.swapchain, None);
    }

    Renderer {
      window: self.window,
      entry: self.entry,
      instance: self.instance,
      surface_instance: self.surface_instance,
      surface: self.surface,
      physical_device: self.physical_device,
      graphics_queue_family_index: self.graphics_queue_family_index,
      present_queue_family_index: self.present_queue_family_index,
      device: self.device,
      graphics_queue: self.graphics_queue,
      present_queue: self.present_queue,
      graphics_command_pools: self.graphics_command_pools,
      graphics_command_buffers: self.graphics_command_buffers,
      swapchain_device: self.swapchain_device,
      render_pass: self.render_pass,
      image_avail_semaphores: self.image_avail_semaphores,
      render_finished_semaphores: self.render_finished_semaphores,
      in_flight_fences: self.in_flight_fences,
      state: Creating { rect_pipeline },
    }
  }

  pub(crate) fn drop(self) {
    let device = self.device.get();

    unsafe {
      device.device_wait_idle().unwrap();
      self.state.rect_pipeline.drop();

      self
        .state
        .swapchain_framebuffers
        .into_iter()
        .for_each(|framebuffer| device.destroy_framebuffer(framebuffer, None));

      self
        .state
        .swapchain_image_views
        .into_iter()
        .for_each(|image_view| device.destroy_image_view(image_view, None));

      self
        .swapchain_device
        .destroy_swapchain(self.state.swapchain, None);

      self
        .in_flight_fences
        .into_iter()
        .for_each(|fence| device.destroy_fence(fence, None));

      self
        .render_finished_semaphores
        .into_iter()
        .for_each(|semaphore| device.destroy_semaphore(semaphore, None));

      self
        .image_avail_semaphores
        .into_iter()
        .for_each(|semaphore| device.destroy_semaphore(semaphore, None));

      device.destroy_render_pass(self.render_pass, None);

      self
        .graphics_command_pools
        .into_iter()
        .for_each(|command_pool| device.destroy_command_pool(command_pool, None));

      device.destroy_device(None);
      self.surface_instance.destroy_surface(self.surface, None);
      self.instance.destroy_instance(None);
    }
  }
}
