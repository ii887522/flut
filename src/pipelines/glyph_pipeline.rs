use crate::shaders::{GlyphFragShader, GlyphVertShader};
use ash::{
  Device,
  vk::{
    self, BlendFactor, BlendOp, ColorComponentFlags, CullModeFlags, DeviceAddress, Extent2D,
    FrontFace, GraphicsPipelineCreateInfo, Offset2D, Pipeline, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineCreateFlags,
    PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode,
    PrimitiveTopology, Rect2D, RenderPass, SampleCountFlags, Viewport,
  },
};
use std::rc::Rc;

#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub(crate) struct PushConstant {
  pub(crate) camera_position: (f32, f32),
  pub(crate) camera_size: (f32, f32),
  pub(crate) mesh_buffer_addr: DeviceAddress,
}

pub(crate) struct GlyphPipeline {
  device: Rc<Device>,
  pub(crate) pipeline: Pipeline,
}

impl GlyphPipeline {
  pub(crate) fn new(
    device: Rc<Device>,
    surface_extent: Extent2D,
    vert_shader: &GlyphVertShader<'_>,
    frag_shader: &GlyphFragShader<'_>,
    layout: PipelineLayout,
    render_pass: RenderPass,
    base_pipeline: Option<&GlyphPipeline>,
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
      blend_enable: vk::TRUE,
      src_color_blend_factor: BlendFactor::SRC_ALPHA,
      dst_color_blend_factor: BlendFactor::ONE_MINUS_SRC_ALPHA,
      color_blend_op: BlendOp::ADD,
      src_alpha_blend_factor: BlendFactor::ONE,
      dst_alpha_blend_factor: BlendFactor::ZERO,
      alpha_blend_op: BlendOp::ADD,
      color_write_mask: ColorComponentFlags::RGBA,
    };

    let color_blend_state_create_info = PipelineColorBlendStateCreateInfo {
      attachment_count: 1,
      p_attachments: &color_blend_attachment_state_create_info,
      ..Default::default()
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
      pipeline: pipelines[0],
    }
  }
}

impl Drop for GlyphPipeline {
  fn drop(&mut self) {
    unsafe { self.device.destroy_pipeline(self.pipeline, None) };
  }
}
