use crate::shaders::{RectFragShader, RectVertShader};
use ash::{
  Device,
  vk::{
    ColorComponentFlags, CullModeFlags, Extent2D, FrontFace, GraphicsPipelineCreateInfo, Offset2D,
    Pipeline, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineCreateFlags, PipelineInputAssemblyStateCreateInfo, PipelineLayout,
    PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
    PrimitiveTopology, PushConstantRange, Rect2D, RenderPass, SampleCountFlags, ShaderStageFlags,
    Viewport,
  },
};
use std::{mem, rc::Rc};

#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub(crate) struct PushConstant {
  pub(crate) camera_size: (f32, f32),
}

pub(crate) struct RectPipeline {
  device: Rc<Device>,
  pub(crate) layout: PipelineLayout,
  pub(crate) pipeline: Pipeline,
}

impl RectPipeline {
  pub(crate) fn new(
    device: Rc<Device>,
    surface_extent: Extent2D,
    vert_shader: &RectVertShader<'_>,
    frag_shader: &RectFragShader<'_>,
    render_pass: RenderPass,
    base_pipeline: Option<&RectPipeline>,
  ) -> Self {
    let input_assembly_state_create_info = PipelineInputAssemblyStateCreateInfo {
      topology: PrimitiveTopology::TRIANGLE_LIST,
      ..Default::default()
    };

    let viewport = Viewport {
      x: 0.0,
      y: 0.0,
      width: surface_extent.width as _,
      height: surface_extent.height as _,
      min_depth: 0.0,
      max_depth: 1.0,
    };

    let scissor = Rect2D {
      offset: Offset2D { x: 0, y: 0 },
      extent: surface_extent,
    };

    let viewport_state_create_info = PipelineViewportStateCreateInfo {
      viewport_count: 1,
      p_viewports: &viewport,
      scissor_count: 1,
      p_scissors: &scissor,
      ..Default::default()
    };

    let rasterization_state_create_info = PipelineRasterizationStateCreateInfo {
      polygon_mode: PolygonMode::FILL,
      cull_mode: CullModeFlags::NONE,
      front_face: FrontFace::CLOCKWISE,
      line_width: 1.0,
      ..Default::default()
    };

    let multisample_state_create_info = PipelineMultisampleStateCreateInfo {
      rasterization_samples: SampleCountFlags::TYPE_1,
      ..Default::default()
    };

    let color_blend_attachment_state_create_info = PipelineColorBlendAttachmentState {
      color_write_mask: ColorComponentFlags::RGBA,
      ..Default::default()
    };

    let color_blend_state_create_info = PipelineColorBlendStateCreateInfo {
      attachment_count: 1,
      p_attachments: &color_blend_attachment_state_create_info,
      ..Default::default()
    };

    let push_const_range = PushConstantRange {
      stage_flags: ShaderStageFlags::VERTEX,
      size: mem::size_of::<PushConstant>() as _,
      ..Default::default()
    };

    let layout_create_info = PipelineLayoutCreateInfo {
      push_constant_range_count: 1,
      p_push_constant_ranges: &push_const_range,
      ..Default::default()
    };

    let layout = unsafe {
      device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap()
    };

    let shader_stage_create_infos = [
      vert_shader.shader_stage_create_info,
      frag_shader.shader_stage_create_info,
    ];

    let pipeline_create_info = GraphicsPipelineCreateInfo {
      flags: if base_pipeline.is_some() {
        PipelineCreateFlags::DERIVATIVE | PipelineCreateFlags::ALLOW_DERIVATIVES
      } else {
        PipelineCreateFlags::ALLOW_DERIVATIVES
      },
      stage_count: shader_stage_create_infos.len() as _,
      p_stages: shader_stage_create_infos.as_ptr(),
      p_vertex_input_state: &vert_shader.vert_input_stage_create_info,
      p_input_assembly_state: &input_assembly_state_create_info,
      p_viewport_state: &viewport_state_create_info,
      p_rasterization_state: &rasterization_state_create_info,
      p_multisample_state: &multisample_state_create_info,
      p_color_blend_state: &color_blend_state_create_info,
      layout,
      render_pass,
      subpass: 0,
      base_pipeline_handle: if let Some(base_pipeline) = base_pipeline {
        base_pipeline.pipeline
      } else {
        Pipeline::null()
      },
      base_pipeline_index: -1,
      ..Default::default()
    };

    let pipelines = unsafe {
      device
        .create_graphics_pipelines(PipelineCache::null(), &[pipeline_create_info], None)
        .unwrap()
    };

    Self {
      device,
      layout,
      pipeline: pipelines[0],
    }
  }
}

impl Drop for RectPipeline {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_pipeline(self.pipeline, None);
      self.device.destroy_pipeline_layout(self.layout, None);
    }
  }
}
