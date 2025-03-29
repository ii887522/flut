use crate::{
  pipelines::BasicPipeline,
  shaders::{BasicFragShader, BasicVertShader, BasicVertex},
  string_slice::StringSlice,
};
use ash::{
  Device, Entry, Instance,
  khr::{surface, swapchain},
  vk::{
    self, ApplicationInfo, BindBufferMemoryInfo, Buffer, BufferCreateInfo,
    BufferMemoryRequirementsInfo2, BufferUsageFlags, ClearColorValue, ClearValue, CommandBuffer,
    CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsageFlags,
    CommandPool, CommandPoolCreateInfo, CompositeAlphaFlagsKHR, DeviceCreateInfo,
    DeviceQueueCreateInfo, DeviceQueueInfo2, Extent2D, Fence, FenceCreateFlags, FenceCreateInfo,
    Framebuffer, FramebufferCreateInfo, Handle, Image, ImageAspectFlags, ImageSubresourceRange,
    ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, InstanceCreateFlags,
    InstanceCreateInfo, MemoryRequirements2, Offset2D, PhysicalDevice, PhysicalDeviceProperties2,
    PhysicalDeviceType, PipelineBindPoint, PipelineStageFlags, PresentInfoKHR, PresentModeKHR,
    Queue, QueueFamilyProperties2, QueueFlags, Rect2D, RenderPassBeginInfo, Semaphore,
    SemaphoreCreateInfo, SharingMode, SubmitInfo, SubpassBeginInfo, SubpassContents,
    SubpassEndInfo, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR, ValidationFeatureEnableEXT,
    ValidationFeaturesEXT,
  },
};
use gpu_allocator::{
  AllocationSizes, AllocatorDebugSettings, MemoryLocation,
  vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc},
};
use sdl2::video::Window;
use std::{ffi::c_void, mem, rc::Rc};

const MAX_IN_FLIGHT_FRAME_COUNT: usize = 3;
const MIN_ALLOC_SIZE: u64 = 4 * 1024 * 1024;

const VERTICES: &[BasicVertex] = &[
  BasicVertex {
    position: (-0.5, -0.5),
  },
  BasicVertex {
    position: (0.0, 0.5),
  },
  BasicVertex {
    position: (0.5, -0.25),
  },
];

pub(super) struct VkEngine<'a> {
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
  basic_vert_shader: BasicVertShader<'a>,
  basic_frag_shader: BasicFragShader<'a>,
  memory_allocator: Option<Allocator>,
  vert_buffer: Buffer,
  _vert_buffer_alloc: Allocation,
  command_pool: CommandPool,
  image_avail_semaphores: Vec<Semaphore>,
  render_done_semaphores: Vec<Semaphore>,
  in_flight_fences: Vec<Fence>,
  swapchain: SwapchainKHR,
  swapchain_images: Vec<Image>,
  swapchain_image_views: Vec<ImageView>,
  basic_pipeline: Option<BasicPipeline>,
  swapchain_framebuffers: Vec<Framebuffer>,
  swapchain_command_buffers: Vec<CommandBuffer>,
  frame_index: usize,
}

impl<'a> VkEngine<'a> {
  pub(super) fn new(window: Window, prefer_dgpu: bool) -> Self {
    let entry = unsafe { Entry::load().unwrap() };

    let enabled_layers = StringSlice::from(
      #[cfg(debug_assertions)]
      ["VK_LAYER_KHRONOS_validation"].as_slice(),
      #[cfg(not(debug_assertions))]
      [].as_slice(),
    );

    let mut enabled_extensions = window.vulkan_instance_extensions().unwrap();

    #[cfg(debug_assertions)]
    enabled_extensions.extend([
      vk::EXT_DEBUG_UTILS_NAME.to_str().unwrap(),
      vk::EXT_VALIDATION_FEATURES_NAME.to_str().unwrap(),
    ]);

    enabled_extensions.push(vk::KHR_PORTABILITY_ENUMERATION_NAME.to_str().unwrap());

    let enabled_extensions = StringSlice::from(enabled_extensions.as_slice());

    #[cfg(debug_assertions)]
    let enabled_validation_features = [
      ValidationFeatureEnableEXT::BEST_PRACTICES,
      ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
    ];

    let app_info = ApplicationInfo {
      api_version: vk::make_api_version(0, 1, 4, 0),
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
      enabled_extension_count: enabled_extensions.len() as _,
      pp_enabled_extension_names: enabled_extensions.as_ptr(),

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
    let enabled_exts = [vk::KHR_SWAPCHAIN_NAME];

    let (physical_device, graphics_queue_family_index, present_queue_family_index) = unsafe {
      instance
        .enumerate_physical_devices()
        .unwrap()
        .iter()
        .filter_map(|&physical_device| {
          let ext_props = instance
            .enumerate_device_extension_properties(physical_device)
            .unwrap();

          if !enabled_exts.iter().all(|&enabled_ext| {
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

    let enabled_exts = StringSlice::from(
      enabled_exts
        .map(|enabled_ext| enabled_ext.to_str().unwrap())
        .as_slice(),
    );

    let device_create_info = DeviceCreateInfo {
      queue_create_info_count: queue_create_infos.len() as _,
      p_queue_create_infos: queue_create_infos.as_ptr(),
      enabled_extension_count: enabled_exts.len() as _,
      pp_enabled_extension_names: enabled_exts.as_ptr(),
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

    let basic_vert_shader = BasicVertShader::new(device.clone());
    let basic_frag_shader = BasicFragShader::new(device.clone());

    let mut memory_allocator = Allocator::new(&AllocatorCreateDesc {
      instance: instance.clone(),
      device: (*device).clone(),
      physical_device,
      debug_settings: AllocatorDebugSettings::default(),
      buffer_device_address: false,
      allocation_sizes: AllocationSizes::new(MIN_ALLOC_SIZE, MIN_ALLOC_SIZE),
    })
    .unwrap();

    let vert_buffer_create_info = BufferCreateInfo {
      size: mem::size_of_val(VERTICES) as _,
      usage: BufferUsageFlags::VERTEX_BUFFER,
      sharing_mode: SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let vert_buffer = unsafe {
      device
        .create_buffer(&vert_buffer_create_info, None)
        .unwrap()
    };

    let vert_buffer_mem_req_info = BufferMemoryRequirementsInfo2 {
      buffer: vert_buffer,
      ..Default::default()
    };

    let mut vert_buffer_mem_req = MemoryRequirements2::default();

    unsafe {
      device.get_buffer_memory_requirements2(&vert_buffer_mem_req_info, &mut vert_buffer_mem_req);
    }

    let mut vert_buffer_alloc = memory_allocator
      .allocate(&AllocationCreateDesc {
        name: "vert_buffer",
        requirements: vert_buffer_mem_req.memory_requirements,
        location: MemoryLocation::CpuToGpu,
        linear: true,
        allocation_scheme: AllocationScheme::DedicatedBuffer(vert_buffer),
      })
      .unwrap();

    let bind_vert_buffer_mem_info = BindBufferMemoryInfo {
      buffer: vert_buffer,
      memory: unsafe { vert_buffer_alloc.memory() },
      memory_offset: vert_buffer_alloc.offset(),
      ..Default::default()
    };

    unsafe {
      device
        .bind_buffer_memory2(&[bind_vert_buffer_mem_info])
        .unwrap();
    }

    let mut mapped_vert_buffer_alloc = vert_buffer_alloc.try_as_mapped_slab().unwrap();
    presser::copy_from_slice_to_offset(VERTICES, &mut mapped_vert_buffer_alloc, 0).unwrap();

    let command_pool_create_info = CommandPoolCreateInfo {
      queue_family_index: graphics_queue_family_index as _,
      ..Default::default()
    };

    let command_pool = unsafe {
      device
        .create_command_pool(&command_pool_create_info, None)
        .unwrap()
    };

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
      basic_vert_shader,
      basic_frag_shader,
      memory_allocator: Some(memory_allocator),
      vert_buffer,
      _vert_buffer_alloc: vert_buffer_alloc,
      command_pool,
      image_avail_semaphores,
      render_done_semaphores,
      in_flight_fences,
      swapchain: SwapchainKHR::null(),
      swapchain_images: vec![],
      swapchain_image_views: vec![],
      basic_pipeline: None,
      swapchain_framebuffers: vec![],
      swapchain_command_buffers: vec![],
      frame_index: 0,
    };

    // Create a new swapchain and its dependents during initialization
    this.on_swapchain_suboptimal();
    this
  }

  pub(super) fn draw(&mut self) {
    unsafe {
      let in_flight_fence = self.in_flight_fences[self.frame_index];
      let image_avail_semaphore = self.image_avail_semaphores[self.frame_index];
      let render_done_semaphore = self.render_done_semaphores[self.frame_index];

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

      self.device.reset_fences(&[in_flight_fence]).unwrap();
      let wait_dst_stage_mask = PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
      let swapchain_command_buffer = self.swapchain_command_buffers[swapchain_image_index as usize];

      let queue_submit_info = SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_avail_semaphore,
        p_wait_dst_stage_mask: &wait_dst_stage_mask,
        command_buffer_count: 1,
        p_command_buffers: &swapchain_command_buffer,
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

      if !self.swapchain_command_buffers.is_empty() {
        self
          .device
          .free_command_buffers(self.command_pool, &self.swapchain_command_buffers);
      }

      self.swapchain_framebuffers.iter().for_each(|&framebuffer| {
        self.device.destroy_framebuffer(framebuffer, None);
      });

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

    let basic_pipeline = BasicPipeline::new(
      self.device.clone(),
      surface_extent,
      surface_format.format,
      &self.basic_vert_shader,
      &self.basic_frag_shader,
      self.basic_pipeline.as_ref(),
    );

    if let Some(basic_pipeline) = &self.basic_pipeline {
      basic_pipeline.drop();
    }

    let swapchain_framebuffers = unsafe {
      swapchain_image_views
        .iter()
        .map(|&image_view| {
          let framebuffer_create_info = FramebufferCreateInfo {
            render_pass: basic_pipeline.render_pass,
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

    let swapchain_command_buffer_alloc_info = CommandBufferAllocateInfo {
      command_pool: self.command_pool,
      level: CommandBufferLevel::PRIMARY,
      command_buffer_count: swapchain_framebuffers.len() as _,
      ..Default::default()
    };

    let swapchain_command_buffers = unsafe {
      self
        .device
        .allocate_command_buffers(&swapchain_command_buffer_alloc_info)
        .unwrap()
    };

    let clear_value = ClearValue {
      color: ClearColorValue {
        float32: [0.0, 0.0, 0.0, 1.0],
      },
    };

    swapchain_command_buffers
      .iter()
      .zip(swapchain_framebuffers.iter())
      .for_each(|(&command_buffer, &framebuffer)| {
        let command_buffer_begin_info = CommandBufferBeginInfo {
          flags: CommandBufferUsageFlags::SIMULTANEOUS_USE,
          ..Default::default()
        };

        let render_pass_begin_info = RenderPassBeginInfo {
          render_pass: basic_pipeline.render_pass,
          framebuffer,
          render_area: Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent: surface_extent,
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

        unsafe {
          self
            .device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .unwrap();

          self.device.cmd_begin_render_pass2(
            command_buffer,
            &render_pass_begin_info,
            &subpass_begin_info,
          );

          self.device.cmd_bind_pipeline(
            command_buffer,
            PipelineBindPoint::GRAPHICS,
            basic_pipeline.pipeline,
          );

          self.device.cmd_bind_vertex_buffers2(
            command_buffer,
            0,
            &[self.vert_buffer],
            &[0],
            None,
            None,
          );

          self
            .device
            .cmd_draw(command_buffer, VERTICES.len() as _, 1, 0, 0);

          self
            .device
            .cmd_end_render_pass2(command_buffer, &subpass_end_info);

          self.device.end_command_buffer(command_buffer).unwrap();
        }
      });

    self.swapchain = swapchain;
    self.swapchain_images = swapchain_images;
    self.swapchain_image_views = swapchain_image_views;
    self.basic_pipeline = Some(basic_pipeline);
    self.swapchain_framebuffers = swapchain_framebuffers;
    self.swapchain_command_buffers = swapchain_command_buffers;
  }
}

impl Drop for VkEngine<'_> {
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
      self.device.destroy_buffer(self.vert_buffer, None);
      drop(mem::take(&mut self.memory_allocator));
      self.basic_frag_shader.drop();
      self.basic_vert_shader.drop();

      self.swapchain_framebuffers.iter().for_each(|&framebuffer| {
        self.device.destroy_framebuffer(framebuffer, None);
      });
    }

    self.basic_pipeline.as_ref().unwrap().drop();

    unsafe {
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
