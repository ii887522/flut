use crate::{
  batches::{GlyphBatch, RoundRectBatch},
  collections::{SparseVec, StringSlice},
  consts,
  images::StaticImage,
  models::{AudioReq, Glyph, Icon, Rect, RoundRect, Text},
};
use ash::{
  Device, Entry, Instance,
  khr::{surface, swapchain},
  vk::{
    self, AccessFlags, ApplicationInfo, AttachmentDescription2, AttachmentLoadOp,
    AttachmentReference2, AttachmentStoreOp, ClearColorValue, ClearDepthStencilValue, ClearValue,
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel,
    CommandBufferUsageFlags, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo,
    CompositeAlphaFlagsKHR, DependencyFlags, DescriptorPool, DescriptorPoolCreateInfo,
    DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorType, DeviceCreateInfo,
    DeviceQueueCreateInfo, DeviceQueueInfo2, Extent2D, Fence, FenceCreateFlags, FenceCreateInfo,
    Format, Framebuffer, FramebufferCreateInfo, Handle, Image, ImageAspectFlags, ImageLayout,
    ImageMemoryBarrier, ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo,
    ImageViewType, InstanceCreateFlags, InstanceCreateInfo, Offset2D, PhysicalDevice,
    PhysicalDeviceProperties2, PhysicalDeviceType, PhysicalDeviceVulkan12Features,
    PipelineBindPoint, PipelineStageFlags, PresentInfoKHR, PresentModeKHR, Queue,
    QueueFamilyProperties2, QueueFlags, Rect2D, RenderPass, RenderPassBeginInfo,
    RenderPassCreateInfo2, SampleCountFlags, Semaphore, SemaphoreCreateInfo, SharingMode,
    SubmitInfo, SubpassBeginInfo, SubpassContents, SubpassDependency2, SubpassDescription2,
    SubpassEndInfo, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR, ValidationFeatureEnableEXT,
    ValidationFeaturesEXT,
  },
};
use gpu_allocator::{
  AllocationSizes, AllocatorDebugSettings,
  vulkan::{Allocator, AllocatorCreateDesc},
};
use rayon::prelude::*;
use sdl2::{ttf::Sdl2TtfContext, video::Window};
use std::{
  cell::RefCell,
  collections::HashSet,
  mem,
  rc::Rc,
  sync::{Arc, mpsc::Sender},
};

pub struct Engine<'a> {
  window: Window,
  _entry: Entry,
  instance: Instance,
  surface: SurfaceKHR,
  surface_instance: surface::Instance,
  physical_device: PhysicalDevice,
  graphics_queue_family_index: u32,
  present_queue_family_index: u32,
  transfer_queue_family_index: u32,
  device: Arc<Device>,
  graphics_queue: Queue,
  present_queue: Queue,
  transfer_queue: Queue,
  swapchain_device: swapchain::Device,
  memory_allocator: Option<Rc<RefCell<Allocator>>>,
  glyph_batch: Option<GlyphBatch<'a>>,
  round_rect_batch: Option<RoundRectBatch<'a>>,
  graphics_command_pool: CommandPool,
  transfer_command_pool: CommandPool,
  graphics_command_buffers: Vec<CommandBuffer>,
  descriptor_pool: DescriptorPool,
  descriptor_sets: Vec<DescriptorSet>,
  image_avail_semaphores: Vec<Semaphore>,
  render_done_semaphores: Vec<Semaphore>,
  in_flight_fences: Vec<Fence>,
  swapchain: SwapchainKHR,
  swapchain_images: Vec<Image>,
  swapchain_image_views: Vec<ImageView>,
  depth_images: Vec<StaticImage>,
  render_pass: RenderPass,
  swapchain_framebuffers: Vec<Framebuffer>,
  surface_extent: Extent2D,
  frame_index: usize,
  text_ids: SparseVec<Vec<u16>>,
  audio_tx: Sender<AudioReq>,
}

pub struct DrawableCaps {
  pub glyph_cap: usize,
  pub round_rect_cap: usize,
}

impl Default for DrawableCaps {
  fn default() -> Self {
    Self {
      glyph_cap: 3000,
      round_rect_cap: 100,
    }
  }
}

impl<'a> Engine<'a> {
  pub(super) fn new(
    ttf: &'a Sdl2TtfContext,
    window: Window,
    audio_tx: Sender<AudioReq>,
    prefer_dgpu: bool,
    drawable_caps: DrawableCaps,
  ) -> Self {
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
      p_next: &validation_features as *const _ as *const _,

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

    let (
      physical_device,
      graphics_queue_family_index,
      present_queue_family_index,
      transfer_queue_family_index,
    ) = unsafe {
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

          let (transfer_queue_family_index, _) = queue_family_properties
            .iter()
            .enumerate()
            .filter(|&(_, queue_family_properties)| {
              queue_family_properties
                .queue_family_properties
                .queue_flags
                .contains(QueueFlags::TRANSFER)
            })
            .min_by_key(|&(_, queue_family_properties)| {
              queue_family_properties.queue_family_properties.queue_flags
            })
            .unwrap();

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
            return Some((
              physical_device,
              queue_family_index,
              queue_family_index,
              transfer_queue_family_index,
            ));
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
            transfer_queue_family_index,
          ))
        })
        .min_by_key(|&(physical_device, _, _, _)| {
          let mut physical_device_properties = PhysicalDeviceProperties2::default();

          instance
            .get_physical_device_properties2(physical_device, &mut physical_device_properties);

          match physical_device_properties.properties.device_type {
            PhysicalDeviceType::INTEGRATED_GPU => {
              if prefer_dgpu {
                1
              } else {
                0
              }
            }
            PhysicalDeviceType::DISCRETE_GPU => {
              if prefer_dgpu {
                0
              } else {
                1
              }
            }
            PhysicalDeviceType::VIRTUAL_GPU => 2,
            PhysicalDeviceType::CPU => 3,
            _ => 4,
          }
        })
        .unwrap()
    };

    let queue_priorities = [1.0];

    let queue_create_infos = HashSet::<_>::from_iter([
      graphics_queue_family_index,
      present_queue_family_index,
      transfer_queue_family_index,
    ])
    .iter()
    .map(|&queue_family_index| DeviceQueueCreateInfo {
      queue_family_index: queue_family_index as _,
      queue_count: 1,
      p_queue_priorities: queue_priorities.as_ptr(),
      ..Default::default()
    })
    .collect::<Vec<_>>();

    let enabled_device_exts = StringSlice::from(
      enabled_device_exts
        .map(|enabled_ext| enabled_ext.to_str().unwrap())
        .as_slice(),
    );

    let device_vk_12_features = PhysicalDeviceVulkan12Features {
      buffer_device_address: vk::TRUE,
      ..Default::default()
    };

    let device_create_info = DeviceCreateInfo {
      queue_create_info_count: queue_create_infos.len() as _,
      p_queue_create_infos: queue_create_infos.as_ptr(),
      enabled_extension_count: enabled_device_exts.len() as _,
      pp_enabled_extension_names: enabled_device_exts.as_ptr(),
      p_next: &device_vk_12_features as *const _ as *const _,
      ..Default::default()
    };

    let device = unsafe {
      Arc::new(
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

    let transfer_queue_info = DeviceQueueInfo2 {
      queue_family_index: transfer_queue_family_index as _,
      queue_index: 0,
      ..Default::default()
    };

    let graphics_queue = unsafe { device.get_device_queue2(&graphics_queue_info) };
    let present_queue = unsafe { device.get_device_queue2(&present_queue_info) };
    let transfer_queue = unsafe { device.get_device_queue2(&transfer_queue_info) };

    let swapchain_device = swapchain::Device::new(&instance, &device);

    let memory_allocator = Rc::new(RefCell::new(
      Allocator::new(&AllocatorCreateDesc {
        instance: instance.clone(),
        device: (*device).clone(),
        physical_device,
        debug_settings: AllocatorDebugSettings::default(),
        buffer_device_address: true,
        allocation_sizes: AllocationSizes::new(consts::MIN_ALLOC_SIZE, consts::MIN_ALLOC_SIZE),
      })
      .unwrap(),
    ));

    let graphics_command_pool_create_info = CommandPoolCreateInfo {
      flags: CommandPoolCreateFlags::TRANSIENT | CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
      queue_family_index: graphics_queue_family_index as _,
      ..Default::default()
    };

    let transfer_command_pool_create_info = CommandPoolCreateInfo {
      queue_family_index: transfer_queue_family_index as _,
      ..Default::default()
    };

    let graphics_command_pool = unsafe {
      device
        .create_command_pool(&graphics_command_pool_create_info, None)
        .unwrap()
    };

    let transfer_command_pool = unsafe {
      device
        .create_command_pool(&transfer_command_pool_create_info, None)
        .unwrap()
    };

    let glyph_batch = GlyphBatch::new(
      device.clone(),
      memory_allocator.clone(),
      transfer_command_pool,
      ttf,
      drawable_caps.glyph_cap,
    );

    let round_rect_batch = RoundRectBatch::new(
      device.clone(),
      memory_allocator.clone(),
      drawable_caps.round_rect_cap,
    );

    let graphics_command_buffer_alloc_info = CommandBufferAllocateInfo {
      command_pool: graphics_command_pool,
      level: CommandBufferLevel::PRIMARY,
      command_buffer_count: consts::MAX_IN_FLIGHT_FRAME_COUNT,
      ..Default::default()
    };

    let transfer_command_buffer_alloc_info = CommandBufferAllocateInfo {
      command_pool: transfer_command_pool,
      level: CommandBufferLevel::PRIMARY,
      command_buffer_count: 1,
      ..Default::default()
    };

    let graphics_command_buffers = unsafe {
      device
        .allocate_command_buffers(&graphics_command_buffer_alloc_info)
        .unwrap()
    };

    let transfer_command_buffer = unsafe {
      device
        .allocate_command_buffers(&transfer_command_buffer_alloc_info)
        .unwrap()[0]
    };

    let transfer_command_buffer_begin_info = CommandBufferBeginInfo {
      flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    let transfer_queue_submit_info = SubmitInfo {
      command_buffer_count: 1,
      p_command_buffers: &transfer_command_buffer,
      ..Default::default()
    };

    let read_image_memory_barrier = ImageMemoryBarrier {
      src_access_mask: AccessFlags::empty(),
      dst_access_mask: AccessFlags::SHADER_READ,
      old_layout: ImageLayout::UNDEFINED,
      new_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
      src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
      image: glyph_batch.icon_atlas.image.images[glyph_batch.icon_atlas.image.image_index],
      subresource_range: ImageSubresourceRange {
        aspect_mask: ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
      },
      ..Default::default()
    };

    unsafe {
      device
        .begin_command_buffer(transfer_command_buffer, &transfer_command_buffer_begin_info)
        .unwrap();

      crate::record_copy_buffer_to_image_commands(
        device.clone(),
        transfer_command_buffer,
        glyph_batch.font_atlas.image.staging_buffer,
        glyph_batch.font_atlas.image.image,
        &glyph_batch.font_atlas.buffer_image_copies,
        graphics_queue_family_index as _,
        transfer_queue_family_index as _,
        ImageLayout::UNDEFINED,
      );

      device.cmd_pipeline_barrier(
        transfer_command_buffer,
        PipelineStageFlags::TOP_OF_PIPE,
        PipelineStageFlags::FRAGMENT_SHADER,
        DependencyFlags::empty(),
        &[],
        &[],
        &[read_image_memory_barrier],
      );

      device.end_command_buffer(transfer_command_buffer).unwrap();

      device
        .queue_submit(transfer_queue, &[transfer_queue_submit_info], Fence::null())
        .unwrap();
    }

    let descriptor_pool_sizes = [
      DescriptorPoolSize {
        ty: DescriptorType::SAMPLER,
        descriptor_count: 2,
      },
      DescriptorPoolSize {
        ty: DescriptorType::SAMPLED_IMAGE,
        descriptor_count: 4,
      },
    ];

    let descriptor_pool_create_info = DescriptorPoolCreateInfo {
      max_sets: 2,
      pool_size_count: descriptor_pool_sizes.len() as _,
      p_pool_sizes: descriptor_pool_sizes.as_ptr(),
      ..Default::default()
    };

    let descriptor_pool = unsafe {
      device
        .create_descriptor_pool(&descriptor_pool_create_info, None)
        .unwrap()
    };

    let descriptor_set_layouts =
      [glyph_batch.descriptor_set_layout; consts::DYNAMIC_IMAGE_COUNT as _];

    let descriptor_set_alloc_info = DescriptorSetAllocateInfo {
      descriptor_pool,
      descriptor_set_count: descriptor_set_layouts.len() as _,
      p_set_layouts: descriptor_set_layouts.as_ptr(),
      ..Default::default()
    };

    let descriptor_sets = unsafe {
      device
        .allocate_descriptor_sets(&descriptor_set_alloc_info)
        .unwrap()
    };

    glyph_batch.init_descriptor_sets(&descriptor_sets);

    let (image_avail_semaphores, (render_done_semaphores, in_flight_fences)): (
      Vec<Semaphore>,
      (Vec<Semaphore>, Vec<Fence>),
    ) = (0..consts::MAX_IN_FLIGHT_FRAME_COUNT)
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
      transfer_queue_family_index: transfer_queue_family_index as _,
      device,
      graphics_queue,
      present_queue,
      transfer_queue,
      swapchain_device,
      glyph_batch: Some(glyph_batch),
      round_rect_batch: Some(round_rect_batch),
      memory_allocator: Some(memory_allocator),
      graphics_command_pool,
      transfer_command_pool,
      graphics_command_buffers,
      descriptor_pool,
      descriptor_sets,
      image_avail_semaphores,
      render_done_semaphores,
      in_flight_fences,
      swapchain: SwapchainKHR::null(),
      swapchain_images: vec![],
      swapchain_image_views: vec![],
      depth_images: vec![],
      render_pass: RenderPass::null(),
      swapchain_framebuffers: vec![],
      surface_extent: Extent2D::default(),
      frame_index: 0,
      text_ids: SparseVec::new(),
      audio_tx,
    };

    // Create a new swapchain and its dependents during initialization
    this.on_swapchain_suboptimal();

    // After initialized all the Vulkan objects, then only display the window to avoid display black screen during startup
    this.window.show();

    unsafe {
      this.device.queue_wait_idle(transfer_queue).unwrap();

      this
        .device
        .free_command_buffers(transfer_command_pool, &[transfer_command_buffer]);

      let glyph_batch = this.glyph_batch.as_mut().unwrap();
      glyph_batch.font_atlas.image.drop_staging();
    }

    this
  }

  pub(super) fn draw(&mut self) {
    unsafe {
      let glyph_batch = self.glyph_batch.as_mut().unwrap();

      let is_drawing_icon_atlas = glyph_batch.icon_atlas.draw(
        self.transfer_queue,
        self.graphics_queue_family_index,
        self.transfer_queue_family_index,
      );

      let command_buffer = self.graphics_command_buffers[self.frame_index];
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

      let clear_values = [
        ClearValue {
          color: ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
          },
        },
        ClearValue {
          depth_stencil: ClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
          },
        },
      ];

      let render_pass_begin_info = RenderPassBeginInfo {
        render_pass: self.render_pass,
        framebuffer: swapchain_framebuffer,
        render_area: Rect2D {
          offset: Offset2D { x: 0, y: 0 },
          extent: self.surface_extent,
        },
        clear_value_count: clear_values.len() as _,
        p_clear_values: clear_values.as_ptr(),
        ..Default::default()
      };

      let subpass_begin_info = SubpassBeginInfo {
        contents: SubpassContents::INLINE,
        ..Default::default()
      };

      let subpass_end_info = SubpassEndInfo::default();
      let pixel_size = self.window.size().0 as f32 / self.window.vulkan_drawable_size().0 as f32;

      self
        .device
        .begin_command_buffer(command_buffer, &command_buffer_begin_info)
        .unwrap();

      self.device.cmd_begin_render_pass2(
        command_buffer,
        &render_pass_begin_info,
        &subpass_begin_info,
      );

      glyph_batch.record_draw_commands(
        command_buffer,
        self.descriptor_sets[glyph_batch.icon_atlas.image.image_index],
        self.surface_extent,
        pixel_size,
      );

      self
        .round_rect_batch
        .as_ref()
        .unwrap()
        .record_draw_commands(command_buffer, self.surface_extent, pixel_size);

      self
        .device
        .cmd_end_render_pass2(command_buffer, &subpass_end_info);

      self.device.end_command_buffer(command_buffer).unwrap();
      self.device.reset_fences(&[in_flight_fence]).unwrap();

      let mut wait_semaphores = vec![image_avail_semaphore];
      let mut wait_dst_stage_mask = vec![PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

      if is_drawing_icon_atlas {
        wait_semaphores.push(glyph_batch.icon_atlas.image.get_draw_done_semaphore());
        wait_dst_stage_mask.push(PipelineStageFlags::FRAGMENT_SHADER);
      }

      glyph_batch
        .icon_atlas
        .image
        .set_render_done_semaphore(render_done_semaphore);

      let queue_submit_info = SubmitInfo {
        wait_semaphore_count: wait_semaphores.len() as _,
        p_wait_semaphores: wait_semaphores.as_ptr(),
        p_wait_dst_stage_mask: wait_dst_stage_mask.as_ptr(),
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
      self.depth_images.clear();

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

    let memory_allocator = self.memory_allocator.as_ref().unwrap();

    let depth_images = swapchain_image_views
      .iter()
      .enumerate()
      .map(|(index, _)| {
        StaticImage::new(
          self.device.clone(),
          memory_allocator.clone(),
          &format!("depth_image_{index}"),
          Format::D24_UNORM_S8_UINT,
          (surface_extent.width, surface_extent.height),
          ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
          ImageAspectFlags::DEPTH,
          &[],
        )
      })
      .collect::<Vec<_>>();

    let attachment_descs = [
      AttachmentDescription2 {
        format: surface_format.format,
        samples: SampleCountFlags::TYPE_1,
        load_op: AttachmentLoadOp::CLEAR,
        store_op: AttachmentStoreOp::STORE,
        stencil_load_op: AttachmentLoadOp::DONT_CARE,
        stencil_store_op: AttachmentStoreOp::DONT_CARE,
        initial_layout: ImageLayout::UNDEFINED,
        final_layout: ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
      },
      AttachmentDescription2 {
        format: Format::D24_UNORM_S8_UINT,
        samples: SampleCountFlags::TYPE_1,
        load_op: AttachmentLoadOp::CLEAR,
        store_op: AttachmentStoreOp::DONT_CARE,
        stencil_load_op: AttachmentLoadOp::DONT_CARE,
        stencil_store_op: AttachmentStoreOp::DONT_CARE,
        initial_layout: ImageLayout::UNDEFINED,
        final_layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
      },
    ];

    let color_attachment_ref = AttachmentReference2 {
      attachment: 0,
      layout: ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
      ..Default::default()
    };

    let depth_attachment_ref = AttachmentReference2 {
      attachment: 1,
      layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
      ..Default::default()
    };

    let subpass_desc = SubpassDescription2 {
      pipeline_bind_point: PipelineBindPoint::GRAPHICS,
      color_attachment_count: 1,
      p_color_attachments: &color_attachment_ref,
      p_depth_stencil_attachment: &depth_attachment_ref,
      ..Default::default()
    };

    let subpass_dep = SubpassDependency2 {
      src_subpass: vk::SUBPASS_EXTERNAL,
      dst_subpass: 0,
      src_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
      src_access_mask: AccessFlags::empty(),
      dst_stage_mask: PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
      dst_access_mask: AccessFlags::COLOR_ATTACHMENT_WRITE
        | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
      dependency_flags: DependencyFlags::BY_REGION,
      ..Default::default()
    };

    let render_pass_create_info = RenderPassCreateInfo2 {
      attachment_count: attachment_descs.len() as _,
      p_attachments: attachment_descs.as_ptr(),
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

    let glyph_batch = self.glyph_batch.as_mut().unwrap();
    let round_rect_batch = self.round_rect_batch.as_mut().unwrap();
    glyph_batch.on_swapchain_suboptimal(surface_extent, render_pass);
    round_rect_batch.on_swapchain_suboptimal(surface_extent, render_pass);

    let swapchain_framebuffers = unsafe {
      swapchain_image_views
        .iter()
        .zip(depth_images.iter())
        .map(|(&image_view, depth_image)| {
          let attachments = [image_view, depth_image.view];

          let framebuffer_create_info = FramebufferCreateInfo {
            render_pass,
            attachment_count: attachments.len() as _,
            p_attachments: attachments.as_ptr(),
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
    self.depth_images = depth_images;
    self.render_pass = render_pass;
    self.swapchain_framebuffers = swapchain_framebuffers;
    self.surface_extent = surface_extent;
  }

  pub const fn get_audio_tx(&self) -> &Sender<AudioReq> {
    &self.audio_tx
  }

  pub fn set_camera_position(&mut self, camera_position: (f32, f32)) {
    self
      .glyph_batch
      .as_mut()
      .unwrap()
      .set_camera_position(camera_position);
  }

  pub fn add_glyph(&mut self, glyph: Glyph) -> u16 {
    self.glyph_batch.as_mut().unwrap().add(glyph)
  }

  pub fn batch_add_glyphs(&mut self, glyphs: Vec<Glyph>) -> Vec<u16> {
    self.glyph_batch.as_mut().unwrap().batch_add(glyphs)
  }

  pub fn update_glyph(&mut self, id: u16, glyph: Glyph) {
    self.glyph_batch.as_mut().unwrap().update(id, glyph);
  }

  pub fn batch_update_glyphs(&self, ids: &[u16], glyphs: Vec<Glyph>) {
    self.glyph_batch.as_ref().unwrap().batch_update(ids, glyphs);
  }

  pub fn remove_glyph(&mut self, id: u16) -> Glyph {
    self.glyph_batch.as_mut().unwrap().remove(id)
  }

  pub fn batch_remove_glyphs(&mut self, ids: &[u16]) {
    self.glyph_batch.as_mut().unwrap().batch_remove(ids);
  }

  pub fn clear_glyphs(&mut self) {
    self.glyph_batch.as_mut().unwrap().clear();
  }

  pub fn add_rect(&mut self, rect: Rect) -> u16 {
    self.add_glyph(rect.into())
  }

  pub fn batch_add_rects(&mut self, rects: Vec<Rect>) -> Vec<u16> {
    self.batch_add_glyphs(rects.into_par_iter().map(Into::into).collect())
  }

  pub fn update_rect(&mut self, id: u16, rect: Rect) {
    self.update_glyph(id, rect.into());
  }

  pub fn batch_update_rects(&self, ids: &[u16], rects: Vec<Rect>) {
    self.batch_update_glyphs(ids, rects.into_par_iter().map(Into::into).collect());
  }

  pub fn remove_rect(&mut self, id: u16) -> Rect {
    self.remove_glyph(id).into()
  }

  pub fn batch_remove_rects(&mut self, ids: &[u16]) {
    self.batch_remove_glyphs(ids);
  }

  pub fn add_text(&mut self, text: Text) -> u16 {
    let glyph_batch = self.glyph_batch.as_ref().unwrap();
    let ids = self.batch_add_glyphs(text.into_glyphs(&glyph_batch.font_atlas));
    self.text_ids.push(ids)
  }

  pub fn remove_text(&mut self, id: u16) {
    let glyph_ids = self.text_ids.remove(id);
    self.batch_remove_glyphs(&glyph_ids);
  }

  pub fn add_round_rect(&mut self, round_rect: RoundRect) -> u16 {
    self.round_rect_batch.as_mut().unwrap().add(round_rect)
  }

  pub fn update_round_rect(&self, id: u16, round_rect: RoundRect) {
    self
      .round_rect_batch
      .as_ref()
      .unwrap()
      .update(id, round_rect);
  }

  pub fn remove_round_rect(&mut self, id: u16) {
    self.round_rect_batch.as_mut().unwrap().remove(id);
  }

  pub fn clear_round_rects(&mut self) {
    self.round_rect_batch.as_mut().unwrap().clear();
  }

  pub fn add_icon(&mut self, icon: Icon) -> u16 {
    let glyph_batch = self.glyph_batch.as_mut().unwrap();
    let glyph = icon.into_glyph(&mut glyph_batch.icon_atlas);
    self.add_glyph(glyph)
  }

  pub fn update_icon(&mut self, id: u16, icon: Icon) {
    let glyph_batch = self.glyph_batch.as_mut().unwrap();
    let glyph = icon.into_glyph(&mut glyph_batch.icon_atlas);
    self.update_glyph(id, glyph);
  }

  pub fn remove_icon(&mut self, id: u16) {
    self.remove_glyph(id);
  }

  pub fn batch_add_icons(&mut self, icons: Vec<Icon>) -> Vec<u16> {
    let glyph_batch = self.glyph_batch.as_mut().unwrap();

    let glyphs = icons
      .into_iter()
      .map(|icon| icon.into_glyph(&mut glyph_batch.icon_atlas))
      .collect();

    self.batch_add_glyphs(glyphs)
  }

  pub fn batch_update_icons(&mut self, ids: &[u16], icons: Vec<Icon>) {
    let glyph_batch = self.glyph_batch.as_mut().unwrap();

    let glyphs = icons
      .into_iter()
      .map(|icon| icon.into_glyph(&mut glyph_batch.icon_atlas))
      .collect();

    self.batch_update_glyphs(ids, glyphs);
  }

  pub fn batch_remove_icons(&mut self, ids: &[u16]) {
    self.batch_remove_glyphs(ids);
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

      self
        .device
        .destroy_descriptor_pool(self.descriptor_pool, None);

      self
        .device
        .destroy_command_pool(self.transfer_command_pool, None);

      self
        .device
        .destroy_command_pool(self.graphics_command_pool, None);

      drop(mem::take(&mut self.round_rect_batch));
      drop(mem::take(&mut self.glyph_batch));
      drop(mem::take(&mut self.memory_allocator));
      self.device.destroy_render_pass(self.render_pass, None);

      self.swapchain_framebuffers.iter().for_each(|&framebuffer| {
        self.device.destroy_framebuffer(framebuffer, None);
      });

      self.depth_images.clear();

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
