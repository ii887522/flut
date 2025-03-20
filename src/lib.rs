#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

mod shaders;

use sdl2::{event::Event, image::LoadSurface, video::Window};
use std::{collections::HashSet, sync::Arc};
use vulkano::{
  Validated, VulkanError, VulkanLibrary,
  buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
  command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassEndInfo, allocator::StandardCommandBufferAllocator,
  },
  device::{
    Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags,
    physical::{PhysicalDevice, PhysicalDeviceType},
  },
  format::ClearValue,
  image::{
    ImageAspects, ImageSubresourceRange, ImageUsage,
    view::{ImageView, ImageViewCreateInfo},
  },
  instance::{
    Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions,
    debug::ValidationFeatureEnable,
  },
  memory::allocator::{
    AllocationCreateInfo, MemoryAllocatePreference, MemoryTypeFilter, StandardMemoryAllocator,
  },
  pipeline::{
    GraphicsPipeline, PipelineCreateFlags, PipelineLayout, PipelineShaderStageCreateInfo,
    graphics::{
      GraphicsPipelineCreateInfo,
      color_blend::{ColorBlendAttachmentState, ColorBlendState},
      input_assembly::InputAssemblyState,
      multisample::MultisampleState,
      rasterization::RasterizationState,
      subpass::PipelineSubpassType,
      vertex_input::{self, VertexDefinition, VertexInputState},
      viewport::{Viewport, ViewportState},
    },
    layout::PipelineDescriptorSetLayoutCreateInfo,
  },
  render_pass::{Framebuffer, FramebufferCreateInfo, Subpass},
  swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo},
  sync::{self, GpuFuture, Sharing},
};

#[derive(BufferContents, vertex_input::Vertex)]
#[repr(C)]
struct Vertex {
  #[format(R32G32_SFLOAT)]
  position: [f32; 2],
}

pub fn run_app(title: &str, width: u32, height: u32) {
  let sdl = sdl2::init().unwrap();

  // Prevent SDL from creating an OpenGL context by itself
  sdl2::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");

  // Fix blurry UI on high DPI displays
  sdl2::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

  let event_subsys = sdl.event().unwrap();
  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window(title, width, height)
    .allow_highdpi()
    .metal_view()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  if let Ok(favicon) = sdl2::surface::Surface::from_file("assets/favicon.png") {
    window.set_icon(favicon);
  }

  // Call window.show() as early as possible to minimize the perceived startup time
  window.show();

  let window_vk_exts = InstanceExtensions::from_iter(window.vulkan_instance_extensions().unwrap());

  let vk_instance = Instance::new(
    VulkanLibrary::new().unwrap(),
    #[cfg(debug_assertions)]
    InstanceCreateInfo {
      enabled_extensions: InstanceExtensions {
        ext_debug_utils: true,
        ext_validation_features: true,
        ..window_vk_exts
      },
      enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_string()],
      enabled_validation_features: vec![
        ValidationFeatureEnable::BestPractices,
        ValidationFeatureEnable::SynchronizationValidation,
      ],
      flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
      ..Default::default()
    },
    #[cfg(not(debug_assertions))]
    InstanceCreateInfo {
      enabled_extensions: window_vk_exts,
      flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
      ..Default::default()
    },
  )
  .unwrap();

  // SAFETY: Be sure not to drop the `window` before the `Surface` or vulkan `Swapchain`!
  // (SIGSEGV otherwise)
  let surface = unsafe { Surface::from_window_ref(vk_instance.clone(), &window).unwrap() };

  let device_exts = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::empty()
  };

  let (physical_device, graphics_queue_family_index, present_queue_family_index) = vk_instance
    .enumerate_physical_devices()
    .unwrap()
    .filter_map(|physical_device| {
      if !physical_device
        .supported_extensions()
        .contains(&device_exts)
      {
        return None;
      }

      if let Some((queue_family_index, _)) = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .find(|&(queue_family_index, queue_family_properties)| {
          queue_family_properties
            .queue_flags
            .contains(QueueFlags::GRAPHICS)
            && physical_device
              .surface_support(queue_family_index as _, &surface)
              .unwrap_or(false)
        })
      {
        return Some((
          physical_device,
          queue_family_index as _,
          queue_family_index as _,
        ));
      }

      let (graphics_queue_family_index, _) = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .find(|&(_, queue_family_properties)| {
          queue_family_properties
            .queue_flags
            .contains(QueueFlags::GRAPHICS)
        })?;

      let (present_queue_family_index, _) = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .find(|&(queue_family_index, _)| {
          physical_device
            .surface_support(queue_family_index as _, &surface)
            .unwrap_or(false)
        })?;

      Some((
        physical_device,
        graphics_queue_family_index as _,
        present_queue_family_index as _,
      ))
    })
    .max_by_key(
      |(physical_device, _, _)| match physical_device.properties().device_type {
        PhysicalDeviceType::DiscreteGpu | PhysicalDeviceType::IntegratedGpu => 3,
        PhysicalDeviceType::VirtualGpu => 2,
        PhysicalDeviceType::Cpu => 1,
        _ => 0,
      },
    )
    .unwrap();

  let (device, queues) = Device::new(
    physical_device.clone(),
    DeviceCreateInfo {
      enabled_extensions: device_exts,
      queue_create_infos: HashSet::<_>::from_iter([
        graphics_queue_family_index,
        present_queue_family_index,
      ])
      .into_iter()
      .map(|queue_family_index| QueueCreateInfo {
        queue_family_index,
        ..Default::default()
      })
      .collect(),
      ..Default::default()
    },
  )
  .unwrap();

  let queues = queues.collect::<Vec<_>>();

  let (graphics_queue, present_queue) = match queues.as_slice() {
    [queue] => (queue, queue),
    [queue_1, queue_2] => {
      if queue_1.queue_family_index() == graphics_queue_family_index {
        (queue_1, queue_2)
      } else {
        (queue_2, queue_1)
      }
    }
    queues => panic!("Unexpected queues length: {}", queues.len()),
  };

  let basic_vs = shaders::basic::load_vs(device.clone()).unwrap();
  let basic_fs = shaders::basic::load_fs(device.clone()).unwrap();
  let basic_vs_main = basic_vs.entry_point("main").unwrap();
  let basic_fs_main = basic_fs.entry_point("main").unwrap();

  let vertex_input_state = <Vertex as vertex_input::Vertex>::per_vertex()
    .definition(&basic_vs_main)
    .unwrap();

  let pipeline_shader_stage_create_infos = [
    PipelineShaderStageCreateInfo::new(basic_vs_main),
    PipelineShaderStageCreateInfo::new(basic_fs_main),
  ];

  let pipeline_layout = PipelineLayout::new(
    device.clone(),
    PipelineDescriptorSetLayoutCreateInfo::from_stages(&pipeline_shader_stage_create_infos)
      .into_pipeline_layout_create_info(device.clone())
      .unwrap(),
  )
  .unwrap();

  let color_blend_state =
    ColorBlendState::with_attachment_states(1, ColorBlendAttachmentState::default());

  let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

  let vertex_buffer = Buffer::from_iter(
    memory_allocator.clone(),
    BufferCreateInfo {
      usage: BufferUsage::VERTEX_BUFFER,
      ..Default::default()
    },
    AllocationCreateInfo {
      memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
      allocate_preference: MemoryAllocatePreference::AlwaysAllocate,
      ..Default::default()
    },
    [
      Vertex {
        position: [-0.5, -0.5],
      },
      Vertex {
        position: [0.0, 0.5],
      },
      Vertex {
        position: [0.5, -0.25],
      },
    ],
  )
  .unwrap();

  let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
    device.clone(),
    Default::default(),
  ));

  // Create a new swapchain and its dependents during initialization
  let (mut swapchain, base_graphics_pipeline, mut command_buffers) = on_swapchain_suboptimal(
    &window,
    surface.clone(),
    physical_device.clone(),
    device.clone(),
    graphics_queue_family_index,
    present_queue_family_index,
    &pipeline_shader_stage_create_infos,
    vertex_input_state.clone(),
    color_blend_state.clone(),
    pipeline_layout.clone(),
    command_buffer_allocator.clone(),
    &vertex_buffer,
    None,
  );

  event_subsys.register_custom_event::<()>().unwrap();
  let event_sender = event_subsys.event_sender();
  let mut event_pump = sdl.event_pump().unwrap();

  for event in event_pump.wait_iter() {
    if let Event::Quit { .. } = event {
      break;
    }

    let (swapchain_image_index, is_suboptimal, acquire_future) =
      match vulkano::swapchain::acquire_next_image(swapchain.clone(), None)
        .map_err(Validated::unwrap)
      {
        Ok(result) => result,
        Err(VulkanError::OutOfDate) => {
          // Swapchain is out of date and cannot be used anymore, need to recreate
          let (new_swapchain, _, new_command_buffers) = on_swapchain_suboptimal(
            &window,
            surface.clone(),
            physical_device.clone(),
            device.clone(),
            graphics_queue_family_index,
            present_queue_family_index,
            &pipeline_shader_stage_create_infos,
            vertex_input_state.clone(),
            color_blend_state.clone(),
            pipeline_layout.clone(),
            command_buffer_allocator.clone(),
            &vertex_buffer,
            Some(base_graphics_pipeline.clone()),
          );

          swapchain = new_swapchain;
          command_buffers = new_command_buffers;

          // Since we failed to acquire swapchain image, ensure the event queue has at least 1 event to trigger acquire swapchain image
          // in the next iteration
          event_sender.push_custom_event(()).unwrap();
          continue;
        }
        Err(err) => panic!("Error acquiring next image: {err}"),
      };

    if is_suboptimal {
      // Swapchain is not optimal, better recreate
      let (new_swapchain, _, new_command_buffers) = on_swapchain_suboptimal(
        &window,
        surface.clone(),
        physical_device.clone(),
        device.clone(),
        graphics_queue_family_index,
        present_queue_family_index,
        &pipeline_shader_stage_create_infos,
        vertex_input_state.clone(),
        color_blend_state.clone(),
        pipeline_layout.clone(),
        command_buffer_allocator.clone(),
        &vertex_buffer,
        Some(base_graphics_pipeline.clone()),
      );

      swapchain = new_swapchain;
      command_buffers = new_command_buffers;

      // Since we failed to acquire swapchain image, ensure the event queue has at least 1 event to trigger acquire swapchain image
      // in the next iteration
      event_sender.push_custom_event(()).unwrap();
      continue;
    }

    match sync::now(device.clone())
      .join(acquire_future)
      .then_execute(
        graphics_queue.clone(),
        command_buffers[swapchain_image_index as usize].clone(),
      )
      .unwrap()
      .then_swapchain_present(
        present_queue.clone(),
        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), swapchain_image_index),
      )
      .flush()
      .map_err(Validated::unwrap)
    {
      Ok(_) => {}
      Err(VulkanError::OutOfDate) => {
        // Swapchain is out of date and cannot be used anymore, need to recreate
        let (new_swapchain, _, new_command_buffers) = on_swapchain_suboptimal(
          &window,
          surface.clone(),
          physical_device.clone(),
          device.clone(),
          graphics_queue_family_index,
          present_queue_family_index,
          &pipeline_shader_stage_create_infos,
          vertex_input_state.clone(),
          color_blend_state.clone(),
          pipeline_layout.clone(),
          command_buffer_allocator.clone(),
          &vertex_buffer,
          Some(base_graphics_pipeline.clone()),
        );

        swapchain = new_swapchain;
        command_buffers = new_command_buffers;

        // Since we failed to acquire swapchain image, ensure the event queue has at least 1 event to trigger acquire swapchain image
        // in the next iteration
        event_sender.push_custom_event(()).unwrap();
        continue;
      }
      Err(err) => panic!("Failed to flush future: {err}"),
    }
  }
}

fn on_swapchain_suboptimal(
  window: &Window,
  surface: Arc<Surface>,
  physical_device: Arc<PhysicalDevice>,
  device: Arc<Device>,
  graphics_queue_family_index: u32,
  present_queue_family_index: u32,
  pipeline_shader_stage_create_infos: &[PipelineShaderStageCreateInfo],
  vertex_input_state: VertexInputState,
  color_blend_state: ColorBlendState,
  pipeline_layout: Arc<PipelineLayout>,
  command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
  vertex_buffer: &Subbuffer<[Vertex]>,
  base_graphics_pipeline: Option<Arc<GraphicsPipeline>>,
) -> (
  Arc<Swapchain>,
  Arc<GraphicsPipeline>,
  Vec<Arc<PrimaryAutoCommandBuffer>>,
) {
  let surface_capabilities = physical_device
    .surface_capabilities(&surface, Default::default())
    .unwrap();

  let min_image_count = surface_capabilities.min_image_count + 1;

  let min_image_count = if let Some(max_image_count) = surface_capabilities.max_image_count {
    min_image_count.min(max_image_count)
  } else {
    min_image_count
  };

  let (surface_format, _) = physical_device
    .surface_formats(&surface, Default::default())
    .unwrap()[0];

  let surface_extent = window.vulkan_drawable_size().into();

  let surface_sharing = if graphics_queue_family_index == present_queue_family_index {
    Sharing::Exclusive
  } else {
    Sharing::Concurrent(vec![graphics_queue_family_index, present_queue_family_index].into())
  };

  let (swapchain, swapchain_images) = Swapchain::new(
    device.clone(),
    surface.clone(),
    SwapchainCreateInfo {
      min_image_count,
      image_format: surface_format,
      image_extent: surface_extent,
      image_usage: ImageUsage::COLOR_ATTACHMENT,
      image_sharing: surface_sharing,
      ..Default::default()
    },
  )
  .unwrap();

  let swapchain_image_views = swapchain_images
    .iter()
    .map(|swapchain_image| {
      ImageView::new(
        swapchain_image.clone(),
        ImageViewCreateInfo {
          format: surface_format,
          subresource_range: ImageSubresourceRange {
            aspects: ImageAspects::COLOR,
            mip_levels: 0..1,
            array_layers: 0..1,
          },
          ..Default::default()
        },
      )
      .unwrap()
    })
    .collect::<Vec<_>>();

  let render_pass = vulkano::single_pass_renderpass!(
    device.clone(),
    attachments: {
      color: {
        format: surface_format,
        samples: 1,
        load_op: Clear,
        store_op: Store
      }
    },
    pass: {
      color: [color],
      depth_stencil: {}
    }
  )
  .unwrap();

  let swapchain_framebuffers = swapchain_image_views
    .iter()
    .map(|swapchain_image_view| {
      Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
          attachments: vec![swapchain_image_view.clone()],
          ..Default::default()
        },
      )
      .unwrap()
    })
    .collect::<Vec<_>>();

  let viewport = Viewport {
    extent: [surface_extent[0] as _, surface_extent[1] as _],
    ..Default::default()
  };

  let viewport_state = ViewportState {
    viewports: [viewport].into(),
    ..Default::default()
  };

  let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

  let graphics_pipeline = if let Some(base_graphics_pipeline) = base_graphics_pipeline {
    GraphicsPipeline::new(
      device.clone(),
      None,
      GraphicsPipelineCreateInfo {
        flags: PipelineCreateFlags::DERIVATIVE,
        viewport_state: Some(viewport_state),
        subpass: Some(PipelineSubpassType::BeginRenderPass(subpass)),
        base_pipeline: Some(base_graphics_pipeline),
        ..GraphicsPipelineCreateInfo::layout(pipeline_layout)
      },
    )
  } else {
    GraphicsPipeline::new(
      device.clone(),
      None,
      GraphicsPipelineCreateInfo {
        flags: PipelineCreateFlags::ALLOW_DERIVATIVES,
        stages: pipeline_shader_stage_create_infos.into(),
        vertex_input_state: Some(vertex_input_state),
        input_assembly_state: Some(InputAssemblyState::default()),
        viewport_state: Some(viewport_state),
        rasterization_state: Some(RasterizationState::default()),
        multisample_state: Some(MultisampleState::default()),
        color_blend_state: Some(color_blend_state),
        subpass: Some(PipelineSubpassType::BeginRenderPass(subpass)),
        ..GraphicsPipelineCreateInfo::layout(pipeline_layout)
      },
    )
  }
  .unwrap();

  let command_buffers = swapchain_framebuffers
    .iter()
    .map(|swapchain_framebuffer| {
      let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator.clone(),
        graphics_queue_family_index,
        CommandBufferUsage::MultipleSubmit,
      )
      .unwrap();

      unsafe {
        builder
          .begin_render_pass(
            RenderPassBeginInfo {
              clear_values: vec![Some(ClearValue::Float([0.0, 0.0, 0.0, 1.0]))],
              ..RenderPassBeginInfo::framebuffer(swapchain_framebuffer.clone())
            },
            SubpassBeginInfo::default(),
          )
          .unwrap()
          .bind_pipeline_graphics(graphics_pipeline.clone())
          .unwrap()
          .bind_vertex_buffers(0, vertex_buffer.clone())
          .unwrap()
          .draw(vertex_buffer.len() as _, 1, 0, 0)
          .unwrap()
          .end_render_pass(SubpassEndInfo::default())
          .unwrap();
      }

      builder.build().unwrap()
    })
    .collect();

  (swapchain, graphics_pipeline, command_buffers)
}
