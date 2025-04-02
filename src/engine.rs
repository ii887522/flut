use crate::{batches::RectBatch, models::Rect, string_slice::StringSlice};
use ash::{
  Device, Entry, Instance,
  khr::{surface, swapchain},
  vk::{
    self, AccessFlags, ApplicationInfo, AttachmentDescription2, AttachmentLoadOp,
    AttachmentReference2, AttachmentStoreOp, ClearColorValue, ClearValue, CommandBuffer,
    CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags,
    CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, CompositeAlphaFlagsKHR,
    DependencyFlags, DeviceCreateInfo, DeviceQueueCreateInfo, DeviceQueueInfo2, Extent2D, Fence,
    FenceCreateFlags, FenceCreateInfo, Framebuffer, FramebufferCreateInfo, Handle, Image,
    ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageUsageFlags, ImageView,
    ImageViewCreateInfo, ImageViewType, InstanceCreateFlags, InstanceCreateInfo, Offset2D,
    PhysicalDevice, PhysicalDeviceProperties2, PhysicalDeviceType, PipelineBindPoint,
    PipelineStageFlags, PresentInfoKHR, PresentModeKHR, Queue, QueueFamilyProperties2, QueueFlags,
    Rect2D, RenderPass, RenderPassBeginInfo, RenderPassCreateInfo2, SampleCountFlags, Semaphore,
    SemaphoreCreateInfo, SharingMode, SubmitInfo, SubpassBeginInfo, SubpassContents,
    SubpassDependency2, SubpassDescription2, SubpassEndInfo, SurfaceKHR, SwapchainCreateInfoKHR,
    SwapchainKHR, ValidationFeatureEnableEXT, ValidationFeaturesEXT,
  },
};
use gpu_allocator::{
  AllocationSizes, AllocatorDebugSettings,
  vulkan::{Allocator, AllocatorCreateDesc},
};
use sdl2::video::Window;
use std::{cell::RefCell, ffi::c_void, mem, rc::Rc};

const MAX_IN_FLIGHT_FRAME_COUNT: u32 = 3;
const MIN_ALLOC_SIZE: u64 = 4 * 1024 * 1024;

pub struct Engine<'a> {
  window: Window,
  _entry: Entry,
  instance: Instance,
  surface: SurfaceKHR,
  surface_instance: surface::Instance,
  physical_device: PhysicalDevice,
  graphics_queue_family_index: u32,
  present_queue_family_index: u32,
  device: Rc<Device>,
  graphics_queue: Queue,
  present_queue: Queue,
  swapchain_device: swapchain::Device,
  memory_allocator: Option<Rc<RefCell<Allocator>>>,
  rect_batch: Option<RectBatch<'a>>,
  command_pool: CommandPool,
  command_buffers: Vec<CommandBuffer>,
  image_avail_semaphores: Vec<Semaphore>,
  render_done_semaphores: Vec<Semaphore>,
  in_flight_fences: Vec<Fence>,
  swapchain: SwapchainKHR,
  swapchain_images: Vec<Image>,
  swapchain_image_views: Vec<ImageView>,
  render_pass: RenderPass,
  swapchain_framebuffers: Vec<Framebuffer>,
  surface_extent: Extent2D,
  frame_index: usize,
}

impl<'a> Engine<'a> {
  pub(super) fn new(window: Window, prefer_dgpu: bool) -> Self {
    #[cfg(all(not(debug_assertions), target_os = "macos"))]
    let entry = ash_molten::load();

    #[cfg(any(debug_assertions, not(target_os = "macos")))]
    let entry = unsafe { Entry::load().unwrap() };

    let enabled_layers = StringSlice::from(
      #[cfg(debug_assertions)]
      ["VK_LAYER_KHRONOS_validation"].as_slice(),
      #[cfg(not(debug_assertions))]
      [].as_slice(),
    );

    let mut enabled_instance_exts = window.vulkan_instance_extensions().unwrap();

    #[cfg(debug_assertions)]
    enabled_instance_exts.extend([
      vk::EXT_DEBUG_UTILS_NAME.to_str().unwrap(),
      vk::EXT_VALIDATION_FEATURES_NAME.to_str().unwrap(),
      vk::KHR_PORTABILITY_ENUMERATION_NAME.to_str().unwrap(),
    ]);

    let enabled_instance_exts = StringSlice::from(enabled_instance_exts.as_slice());

    #[cfg(debug_assertions)]
    let enabled_validation_features = [
      ValidationFeatureEnableEXT::BEST_PRACTICES,
      ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
    ];

    let app_info = ApplicationInfo {
      api_version: vk::make_api_version(0, 1, 2, 0),
      ..Default::default()
    };

    #[cfg(debug_assertions)]
    let validation_features = ValidationFeaturesEXT {
      enabled_validation_feature_count: enabled_validation_features.len() as _,
      p_enabled_validation_features: enabled_validation_features.as_ptr(),
      ..Default::default()
    };

    let instance_create_info = InstanceCreateInfo {
      flags: InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR,
      p_application_info: &app_info,
      enabled_layer_count: enabled_layers.len() as _,
      pp_enabled_layer_names: enabled_layers.as_ptr(),
      enabled_extension_count: enabled_instance_exts.len() as _,
      pp_enabled_extension_names: enabled_instance_exts.as_ptr(),

      #[cfg(debug_assertions)]
      p_next: &validation_features as *const _ as *const c_void,

      ..Default::default()
    };

    let instance = unsafe { entry.create_instance(&instance_create_info, None).unwrap() };

    let surface = SurfaceKHR::from_raw(
      window
        .vulkan_create_surface(instance.handle().as_raw() as _)
        .unwrap(),
    );

    let surface_instance = surface::Instance::new(&entry, &instance);

    let enabled_device_exts = [
      vk::KHR_SWAPCHAIN_NAME,
      #[cfg(target_os = "macos")]
      vk::KHR_PORTABILITY_SUBSET_NAME,
    ];

    let (physical_device, graphics_queue_family_index, present_queue_family_index) = unsafe {
      instance
        .enumerate_physical_devices()
        .unwrap()
        .iter()
        .filter_map(|&physical_device| {
          let ext_props = instance
            .enumerate_device_extension_properties(physical_device)
            .unwrap();

          if !enabled_device_exts.iter().all(|&enabled_ext| {
            ext_props
              .iter()
              .map(|ext_prop| ext_prop.extension_name_as_c_str().unwrap())
              .any(|ext_name| ext_name == enabled_ext)
          }) {
            return None;
          }

          let queue_family_count =
            instance.get_physical_device_queue_family_properties2_len(physical_device);

          let mut queue_family_properties =
            vec![QueueFamilyProperties2::default(); queue_family_count];

          instance.get_physical_device_queue_family_properties2(
            physical_device,
            &mut queue_family_properties,
          );

          if let Some((queue_family_index, _)) = queue_family_properties.iter().enumerate().find(
            |&(queue_family_index, queue_family_properties)| {
              queue_family_properties
                .queue_family_properties
                .queue_flags
                .contains(QueueFlags::GRAPHICS)
                && surface_instance
                  .get_physical_device_surface_support(
                    physical_device,
                    queue_family_index as _,
                    surface,
                  )
                  .unwrap()
            },
          ) {
            return Some((physical_device, queue_family_index, queue_family_index));
          }

          let (graphics_queue_family_index, _) =
            queue_family_properties
              .iter()
              .enumerate()
              .find(|&(_, queue_family_properties)| {
                queue_family_properties
                  .queue_family_properties
                  .queue_flags
                  .contains(QueueFlags::GRAPHICS)
              })?;

          let (present_queue_family_index, _) =
            queue_family_properties
              .iter()
              .enumerate()
              .find(|&(queue_family_index, _)| {
                surface_instance
                  .get_physical_device_surface_support(
                    physical_device,
                    queue_family_index as _,
                    surface,
                  )
                  .unwrap()
              })?;

          Some((
            physical_device,
            graphics_queue_family_index,
            present_queue_family_index,
          ))
        })
        .max_by_key(|&(physical_device, _, _)| {
          let mut physical_device_properties = PhysicalDeviceProperties2::default();

          instance
            .get_physical_device_properties2(physical_device, &mut physical_device_properties);

          match physical_device_properties.properties.device_type {
            PhysicalDeviceType::INTEGRATED_GPU => {
              if prefer_dgpu {
                3
              } else {
                4
              }
            }
            PhysicalDeviceType::DISCRETE_GPU => {
              if prefer_dgpu {
                4
              } else {
                3
              }
            }
            PhysicalDeviceType::VIRTUAL_GPU => 2,
            PhysicalDeviceType::CPU => 1,
            _ => 0,
          }
        })
        .unwrap()
    };

    let queue_priorities = [1.0];

    let mut queue_create_infos = vec![DeviceQueueCreateInfo {
      queue_family_index: graphics_queue_family_index as _,
      queue_count: 1,
      p_queue_priorities: queue_priorities.as_ptr(),
      ..Default::default()
    }];

    if graphics_queue_family_index != present_queue_family_index {
      queue_create_infos.push(DeviceQueueCreateInfo {
        queue_family_index: present_queue_family_index as _,
        queue_count: 1,
        p_queue_priorities: queue_priorities.as_ptr(),
        ..Default::default()
      });
    }

    let enabled_device_exts = StringSlice::from(
      enabled_device_exts
        .map(|enabled_ext| enabled_ext.to_str().unwrap())
        .as_slice(),
    );

    let device_create_info = DeviceCreateInfo {
      queue_create_info_count: queue_create_infos.len() as _,
      p_queue_create_infos: queue_create_infos.as_ptr(),
      enabled_extension_count: enabled_device_exts.len() as _,
      pp_enabled_extension_names: enabled_device_exts.as_ptr(),
      ..Default::default()
    };

    let device = unsafe {
      Rc::new(
        instance
          .create_device(physical_device, &device_create_info, None)
          .unwrap(),
      )
    };

    let graphics_queue_info = DeviceQueueInfo2 {
      queue_family_index: graphics_queue_family_index as _,
      queue_index: 0,
      ..Default::default()
    };

    let present_queue_info = DeviceQueueInfo2 {
      queue_family_index: present_queue_family_index as _,
      queue_index: 0,
      ..Default::default()
    };

    let graphics_queue = unsafe { device.get_device_queue2(&graphics_queue_info) };
    let present_queue = unsafe { device.get_device_queue2(&present_queue_info) };

    let swapchain_device = swapchain::Device::new(&instance, &device);

    let memory_allocator = Rc::new(RefCell::new(
      Allocator::new(&AllocatorCreateDesc {
        instance: instance.clone(),
        device: (*device).clone(),
        physical_device,
        debug_settings: AllocatorDebugSettings::default(),
        buffer_device_address: false,
        allocation_sizes: AllocationSizes::new(MIN_ALLOC_SIZE, MIN_ALLOC_SIZE),
      })
      .unwrap(),
    ));

    let rect_batch = RectBatch::new(device.clone(), memory_allocator.clone());

    unsafe {
      device
        .bind_buffer_memory2(&[
          rect_batch.inst_buffer.bind_buffer_mem_info,
          rect_batch.vert_buffer.bind_staging_buffer_mem_info,
          rect_batch.vert_buffer.bind_buffer_mem_info,
          rect_batch.index_buffer.bind_staging_buffer_mem_info,
          rect_batch.index_buffer.bind_buffer_mem_info,
        ])
        .unwrap();
    }

    let command_pool_create_info = CommandPoolCreateInfo {
      flags: CommandPoolCreateFlags::TRANSIENT | CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
      queue_family_index: graphics_queue_family_index as _, // Graphics queue family implicitly supports transfer operations
      ..Default::default()
    };

    let command_pool = unsafe {
      device
        .create_command_pool(&command_pool_create_info, None)
        .unwrap()
    };

    let command_buffer_alloc_info = CommandBufferAllocateInfo {
      command_pool,
      level: CommandBufferLevel::PRIMARY,
      command_buffer_count: MAX_IN_FLIGHT_FRAME_COUNT,
      ..Default::default()
    };

    let command_buffers = unsafe {
      device
        .allocate_command_buffers(&command_buffer_alloc_info)
        .unwrap()
    };

    let command_buffer_begin_info = CommandBufferBeginInfo {
      flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    let queue_submit_info = SubmitInfo {
      command_buffer_count: 1,
      p_command_buffers: command_buffers.as_ptr(),
      ..Default::default()
    };

    let command_buffer = command_buffers[0];

    unsafe {
      device
        .begin_command_buffer(command_buffer, &command_buffer_begin_info)
        .unwrap();

      rect_batch.record_init_commands(command_buffer);
      device.end_command_buffer(command_buffer).unwrap();

      device
        .queue_submit(graphics_queue, &[queue_submit_info], Fence::null())
        .unwrap();
    }

    let (image_avail_semaphores, (render_done_semaphores, in_flight_fences)): (
      Vec<Semaphore>,
      (Vec<Semaphore>, Vec<Fence>),
    ) = (0..MAX_IN_FLIGHT_FRAME_COUNT)
      .map(|_| {
        let image_avail_semaphore_create_info = SemaphoreCreateInfo::default();
        let render_done_semaphore_create_info = SemaphoreCreateInfo::default();

        let in_flight_fence_create_info = FenceCreateInfo {
          flags: FenceCreateFlags::SIGNALED,
          ..Default::default()
        };

        unsafe {
          (
            device
              .create_semaphore(&image_avail_semaphore_create_info, None)
              .unwrap(),
            (
              device
                .create_semaphore(&render_done_semaphore_create_info, None)
                .unwrap(),
              device
                .create_fence(&in_flight_fence_create_info, None)
                .unwrap(),
            ),
          )
        }
      })
      .unzip();

    let mut this = Self {
      window,
      _entry: entry,
      instance,
      surface,
      surface_instance,
      physical_device,
      graphics_queue_family_index: graphics_queue_family_index as _,
      present_queue_family_index: present_queue_family_index as _,
      device,
      graphics_queue,
      present_queue,
      swapchain_device,
      rect_batch: Some(rect_batch),
      memory_allocator: Some(memory_allocator),
      command_pool,
      command_buffers,
      image_avail_semaphores,
      render_done_semaphores,
      in_flight_fences,
      swapchain: SwapchainKHR::null(),
      swapchain_images: vec![],
      swapchain_image_views: vec![],
      render_pass: RenderPass::null(),
      swapchain_framebuffers: vec![],
      surface_extent: Extent2D::default(),
      frame_index: 0,
    };

    // Create a new swapchain and its dependents during initialization
    this.on_swapchain_suboptimal();

    unsafe {
      this.device.queue_wait_idle(graphics_queue).unwrap();
      let rect_batch = this.rect_batch.as_mut().unwrap();
      rect_batch.index_buffer.drop_staging();
      rect_batch.vert_buffer.drop_staging();
    }

    this
  }

  pub(super) fn draw(&mut self) {
    unsafe {
      let command_buffer = self.command_buffers[self.frame_index];
      let image_avail_semaphore = self.image_avail_semaphores[self.frame_index];
      let render_done_semaphore = self.render_done_semaphores[self.frame_index];
      let in_flight_fence = self.in_flight_fences[self.frame_index];

      self
        .device
        .wait_for_fences(&[in_flight_fence], true, u64::MAX)
        .unwrap();

      // If after acquired a swapchain image found that the swapchain is suboptimal, we still proceed to submit command buffer
      // because the swapchain image is already acquired and need to be presented before call on_swapchain_suboptimal()
      // to ensure no swapchain images are holding by draw() forever causing deadlock
      let (swapchain_image_index, _is_swapchain_suboptimal) =
        match self.swapchain_device.acquire_next_image(
          self.swapchain,
          u64::MAX,
          image_avail_semaphore,
          Fence::null(),
        ) {
          Ok(resp) => resp,
          Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return self.on_swapchain_suboptimal(),
          Err(err) => panic!("{err}"),
        };

      let command_buffer_begin_info = CommandBufferBeginInfo {
        flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
      };

      let swapchain_framebuffer = self.swapchain_framebuffers[swapchain_image_index as usize];

      let clear_value = ClearValue {
        color: ClearColorValue {
          float32: [0.0, 0.0, 0.0, 1.0],
        },
      };

      let render_pass_begin_info = RenderPassBeginInfo {
        render_pass: self.render_pass,
        framebuffer: swapchain_framebuffer,
        render_area: Rect2D {
          offset: Offset2D { x: 0, y: 0 },
          extent: self.surface_extent,
        },
        clear_value_count: 1,
        p_clear_values: &clear_value,
        ..Default::default()
      };

      let subpass_begin_info = SubpassBeginInfo {
        contents: SubpassContents::INLINE,
        ..Default::default()
      };

      let subpass_end_info = SubpassEndInfo::default();

      self
        .device
        .begin_command_buffer(command_buffer, &command_buffer_begin_info)
        .unwrap();

      self.device.cmd_begin_render_pass2(
        command_buffer,
        &render_pass_begin_info,
        &subpass_begin_info,
      );

      self
        .rect_batch
        .as_ref()
        .unwrap()
        .record_draw_commands(command_buffer, self.surface_extent);

      self
        .device
        .cmd_end_render_pass2(command_buffer, &subpass_end_info);

      self.device.end_command_buffer(command_buffer).unwrap();
      self.device.reset_fences(&[in_flight_fence]).unwrap();
      let wait_dst_stage_mask = PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;

      let queue_submit_info = SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_avail_semaphore,
        p_wait_dst_stage_mask: &wait_dst_stage_mask,
        command_buffer_count: 1,
        p_command_buffers: &command_buffer,
        signal_semaphore_count: 1,
        p_signal_semaphores: &render_done_semaphore,
        ..Default::default()
      };

      self
        .device
        .queue_submit(self.graphics_queue, &[queue_submit_info], in_flight_fence)
        .unwrap();

      let present_info = PresentInfoKHR {
        wait_semaphore_count: 1,
        p_wait_semaphores: &render_done_semaphore,
        swapchain_count: 1,
        p_swapchains: &self.swapchain,
        p_image_indices: &swapchain_image_index,
        ..Default::default()
      };

      match self
        .swapchain_device
        .queue_present(self.present_queue, &present_info)
      {
        Ok(false) => {} // Swapchain is optimal
        Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return self.on_swapchain_suboptimal(), // Swapchain is suboptimal
        Err(err) => panic!("{err}"),
      };

      self.frame_index = (self.frame_index + 1) % self.in_flight_fences.len();
    }
  }

  fn on_swapchain_suboptimal(&mut self) {
    let surface_capabilities = unsafe {
      self
        .surface_instance
        .get_physical_device_surface_capabilities(self.physical_device, self.surface)
        .unwrap()
    };

    let surface_extent = if surface_capabilities.current_extent.width != u32::MAX {
      surface_capabilities.current_extent
    } else {
      let (drawable_width, drawable_height) = self.window.vulkan_drawable_size();

      Extent2D {
        width: drawable_width.clamp(
          surface_capabilities.min_image_extent.width,
          surface_capabilities.max_image_extent.width,
        ),
        height: drawable_height.clamp(
          surface_capabilities.min_image_extent.height,
          surface_capabilities.max_image_extent.height,
        ),
      }
    };

    if surface_extent.width == 0 || surface_extent.height == 0 {
      return;
    }

    let min_image_count = surface_capabilities.min_image_count + 1;

    let min_image_count = if surface_capabilities.max_image_count > 0 {
      min_image_count.min(surface_capabilities.max_image_count)
    } else {
      min_image_count
    };

    let surface_format = unsafe {
      self
        .surface_instance
        .get_physical_device_surface_formats(self.physical_device, self.surface)
        .unwrap()[0]
    };

    let surface_sharing_mode =
      if self.graphics_queue_family_index == self.present_queue_family_index {
        SharingMode::EXCLUSIVE
      } else {
        SharingMode::CONCURRENT
      };

    let surface_queue_family_indices = if surface_sharing_mode == SharingMode::EXCLUSIVE {
      vec![]
    } else {
      vec![
        self.graphics_queue_family_index,
        self.present_queue_family_index,
      ]
    };

    unsafe {
      self.device.device_wait_idle().unwrap();

      self.swapchain_framebuffers.iter().for_each(|&framebuffer| {
        self.device.destroy_framebuffer(framebuffer, None);
      });

      self.device.destroy_render_pass(self.render_pass, None);

      self.swapchain_image_views.iter().for_each(|&image_view| {
        self.device.destroy_image_view(image_view, None);
      });

      if !self.swapchain.is_null() {
        self
          .swapchain_device
          .destroy_swapchain(self.swapchain, None);
      }
    }

    let swapchain_create_info = SwapchainCreateInfoKHR {
      surface: self.surface,
      min_image_count,
      image_format: surface_format.format,
      image_color_space: surface_format.color_space,
      image_extent: surface_extent,
      image_array_layers: 1,
      image_usage: ImageUsageFlags::COLOR_ATTACHMENT,
      image_sharing_mode: surface_sharing_mode,
      queue_family_index_count: surface_queue_family_indices.len() as _,
      p_queue_family_indices: surface_queue_family_indices.as_ptr(),
      pre_transform: surface_capabilities.current_transform,
      composite_alpha: CompositeAlphaFlagsKHR::OPAQUE,
      present_mode: PresentModeKHR::FIFO,
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

    let swapchain_image_views = unsafe {
      swapchain_images.iter().map(|&image| {
        let image_view_create_info = ImageViewCreateInfo {
          image,
          view_type: ImageViewType::TYPE_2D,
          format: surface_format.format,
          subresource_range: ImageSubresourceRange {
            aspect_mask: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
          },
          ..Default::default()
        };

        self
          .device
          .create_image_view(&image_view_create_info, None)
          .unwrap()
      })
    }
    .collect::<Vec<_>>();

    let color_attachment_desc = AttachmentDescription2 {
      format: surface_format.format,
      samples: SampleCountFlags::TYPE_1,
      load_op: AttachmentLoadOp::CLEAR,
      store_op: AttachmentStoreOp::STORE,
      stencil_load_op: AttachmentLoadOp::DONT_CARE,
      stencil_store_op: AttachmentStoreOp::DONT_CARE,
      initial_layout: ImageLayout::UNDEFINED,
      final_layout: ImageLayout::PRESENT_SRC_KHR,
      ..Default::default()
    };

    let color_attachment_ref = AttachmentReference2 {
      attachment: 0,
      layout: ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
      ..Default::default()
    };

    let subpass_desc = SubpassDescription2 {
      pipeline_bind_point: PipelineBindPoint::GRAPHICS,
      color_attachment_count: 1,
      p_color_attachments: &color_attachment_ref,
      ..Default::default()
    };

    let subpass_dep = SubpassDependency2 {
      src_subpass: vk::SUBPASS_EXTERNAL,
      dst_subpass: 0,
      src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
      src_access_mask: AccessFlags::empty(),
      dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
      dst_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE,
      dependency_flags: DependencyFlags::BY_REGION,
      ..Default::default()
    };

    let render_pass_create_info = RenderPassCreateInfo2 {
      attachment_count: 1,
      p_attachments: &color_attachment_desc,
      subpass_count: 1,
      p_subpasses: &subpass_desc,
      dependency_count: 1,
      p_dependencies: &subpass_dep,
      ..Default::default()
    };

    let render_pass = unsafe {
      self
        .device
        .create_render_pass2(&render_pass_create_info, None)
        .unwrap()
    };

    let rect_batch = self.rect_batch.as_mut().unwrap();
    rect_batch.on_swapchain_suboptimal(surface_extent, render_pass);

    let swapchain_framebuffers = unsafe {
      swapchain_image_views
        .iter()
        .map(|&image_view| {
          let framebuffer_create_info = FramebufferCreateInfo {
            render_pass,
            attachment_count: 1,
            p_attachments: &image_view,
            width: surface_extent.width,
            height: surface_extent.height,
            layers: 1,
            ..Default::default()
          };

          self
            .device
            .create_framebuffer(&framebuffer_create_info, None)
            .unwrap()
        })
        .collect::<Vec<_>>()
    };

    self.swapchain = swapchain;
    self.swapchain_images = swapchain_images;
    self.swapchain_image_views = swapchain_image_views;
    self.render_pass = render_pass;
    self.swapchain_framebuffers = swapchain_framebuffers;
    self.surface_extent = surface_extent;
  }

  pub fn add_rect(&mut self, rect: Rect) {
    self.rect_batch.as_mut().unwrap().add(rect);
  }
}

impl Drop for Engine<'_> {
  fn drop(&mut self) {
    unsafe {
      self.device.device_wait_idle().unwrap();

      self.in_flight_fences.iter().for_each(|&fence| {
        self.device.destroy_fence(fence, None);
      });

      self.render_done_semaphores.iter().for_each(|&semaphore| {
        self.device.destroy_semaphore(semaphore, None);
      });

      self.image_avail_semaphores.iter().for_each(|&semaphore| {
        self.device.destroy_semaphore(semaphore, None);
      });

      self.device.destroy_command_pool(self.command_pool, None);
      drop(mem::take(&mut self.rect_batch));
      drop(mem::take(&mut self.memory_allocator));
      self.device.destroy_render_pass(self.render_pass, None);

      self.swapchain_framebuffers.iter().for_each(|&framebuffer| {
        self.device.destroy_framebuffer(framebuffer, None);
      });

      self.swapchain_image_views.iter().for_each(|&image_view| {
        self.device.destroy_image_view(image_view, None);
      });

      self
        .swapchain_device
        .destroy_swapchain(self.swapchain, None);

      self.device.destroy_device(None);
      self.surface_instance.destroy_surface(self.surface, None);
      self.instance.destroy_instance(None);
    }
  }
}
