use ash::vk;
use std::{ffi::CString, mem};

const VERT_SHADER_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/spv/rect.vert.spv"));
const FRAG_SHADER_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/spv/rect.frag.spv"));

#[repr(C)]
#[derive(Clone, Copy)]
struct RectPushConsts {
  rect_buffer: vk::DeviceAddress,
  cam_size: (f32, f32),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Rect {
  pub(crate) position: (f32, f32),
  pub(crate) size: (f32, f32),
  pub(crate) color: (f32, f32, f32, f32),
}

pub(crate) struct RectPipeline {
  layout: vk::PipelineLayout,
  pipeline: vk::Pipeline,
}

impl RectPipeline {
  pub(crate) fn new(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain_image_extent: vk::Extent2D,
  ) -> Self {
    let vert_shader_module_create_info = vk::ShaderModuleCreateInfo {
      code_size: VERT_SHADER_CODE.len(),
      p_code: VERT_SHADER_CODE.as_ptr() as *const _,
      ..Default::default()
    };

    let vert_shader_module = unsafe {
      device
        .create_shader_module(&vert_shader_module_create_info, None)
        .unwrap()
    };

    let frag_shader_module_create_info = vk::ShaderModuleCreateInfo {
      code_size: FRAG_SHADER_CODE.len(),
      p_code: FRAG_SHADER_CODE.as_ptr() as *const _,
      ..Default::default()
    };

    let frag_shader_module = unsafe {
      device
        .create_shader_module(&frag_shader_module_create_info, None)
        .unwrap()
    };

    let push_const_ranges = [vk::PushConstantRange {
      stage_flags: vk::ShaderStageFlags::VERTEX,
      offset: 0,
      size: mem::size_of::<RectPushConsts>() as _,
    }];

    let layout_create_info = vk::PipelineLayoutCreateInfo {
      push_constant_range_count: push_const_ranges.len() as _,
      p_push_constant_ranges: push_const_ranges.as_ptr(),
      ..Default::default()
    };

    let layout = unsafe {
      device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap()
    };

    let main_fn_name = CString::new("main").unwrap();

    let shader_stage_create_infos = [
      vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::VERTEX,
        module: vert_shader_module,
        p_name: main_fn_name.as_ptr(),
        ..Default::default()
      },
      vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: frag_shader_module,
        p_name: main_fn_name.as_ptr(),
        ..Default::default()
      },
    ];

    let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::default();

    let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo {
      topology: vk::PrimitiveTopology::TRIANGLE_LIST,
      ..Default::default()
    };

    let viewport = vk::Viewport {
      width: swapchain_image_extent.width as _,
      height: swapchain_image_extent.height as _,
      min_depth: 0.0,
      max_depth: 1.0,
      ..Default::default()
    };

    let scissor = vk::Rect2D {
      extent: swapchain_image_extent,
      ..Default::default()
    };

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
      viewport_count: 1,
      p_viewports: &viewport,
      scissor_count: 1,
      p_scissors: &scissor,
      ..Default::default()
    };

    let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo {
      polygon_mode: vk::PolygonMode::FILL,
      front_face: vk::FrontFace::CLOCKWISE,
      line_width: 1.0,
      ..Default::default()
    };

    let multisample_state = vk::PipelineMultisampleStateCreateInfo {
      rasterization_samples: vk::SampleCountFlags::TYPE_1,
      ..Default::default()
    };

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
      blend_enable: vk::TRUE,
      src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
      dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
      color_blend_op: vk::BlendOp::ADD,
      src_alpha_blend_factor: vk::BlendFactor::ONE,
      dst_alpha_blend_factor: vk::BlendFactor::ZERO,
      alpha_blend_op: vk::BlendOp::ADD,
      color_write_mask: vk::ColorComponentFlags::R
        | vk::ColorComponentFlags::G
        | vk::ColorComponentFlags::B
        | vk::ColorComponentFlags::A,
    }];

    let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo {
      attachment_count: color_blend_attachment_states.len() as _,
      p_attachments: color_blend_attachment_states.as_ptr(),
      ..Default::default()
    };

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
      flags: vk::PipelineCreateFlags::ALLOW_DERIVATIVES,
      stage_count: shader_stage_create_infos.len() as _,
      p_stages: shader_stage_create_infos.as_ptr(),
      p_vertex_input_state: &vertex_input_state_create_info,
      p_input_assembly_state: &input_assembly_state_create_info,
      p_viewport_state: &viewport_state_create_info,
      p_rasterization_state: &rasterization_state_create_info,
      p_multisample_state: &multisample_state,
      p_color_blend_state: &color_blend_state_create_info,
      layout,
      render_pass,
      subpass: 0,
      ..Default::default()
    };

    let pipeline = unsafe {
      device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
        .unwrap()[0]
    };

    unsafe {
      device.destroy_shader_module(frag_shader_module, None);
      device.destroy_shader_module(vert_shader_module, None);
    }

    Self { layout, pipeline }
  }

  pub(crate) const fn get_pipeline(&self) -> vk::Pipeline {
    self.pipeline
  }

  pub(crate) const fn get_pipeline_layout(&self) -> vk::PipelineLayout {
    self.layout
  }

  pub(crate) fn drop(self, device: &ash::Device) {
    unsafe {
      device.destroy_pipeline(self.pipeline, None);
      device.destroy_pipeline_layout(self.layout, None);
    }
  }
}
