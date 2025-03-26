use crate::shaders::{BasicFragShader, BasicVertShader};
use ash::{
  Device,
  vk::{
    AttachmentDescription2, AttachmentLoadOp, AttachmentReference2, AttachmentStoreOp,
    ColorComponentFlags, CullModeFlags, Extent2D, Format, FrontFace, GraphicsPipelineCreateInfo,
    ImageLayout, Offset2D, Pipeline, PipelineBindPoint, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineCreateFlags,
    PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, Rect2D, RenderPass,
    RenderPassCreateInfo2, SampleCountFlags, SubpassDescription2, Viewport,
  },
};
use std::rc::Rc;

pub(crate) struct BasicPipeline {
  device: Rc<Device>,
  layout: PipelineLayout,
  pub(crate) render_pass: RenderPass,
  pub(crate) pipeline: Pipeline,
}

impl BasicPipeline {
  pub(crate) fn new(
    device: Rc<Device>,
    surface_extent: Extent2D,
    surface_format: Format,
    vert_shader: BasicVertShader<'_>,
    frag_shader: BasicFragShader<'_>,
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
      front_face: FrontFace::COUNTER_CLOCKWISE,
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

    let layout_create_info = PipelineLayoutCreateInfo::default();

    let layout = unsafe {
      device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap()
    };

    let color_attachment_desc = AttachmentDescription2 {
      format: surface_format,
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

    let render_pass_create_info = RenderPassCreateInfo2 {
      attachment_count: 1,
      p_attachments: &color_attachment_desc,
      subpass_count: 1,
      p_subpasses: &subpass_desc,
      ..Default::default()
    };

    let render_pass = unsafe {
      device
        .create_render_pass2(&render_pass_create_info, None)
        .unwrap()
    };

    let shader_stage_create_infos = [
      vert_shader.shader_stage_create_info,
      frag_shader.shader_stage_create_info,
    ];

    let pipeline_create_info = GraphicsPipelineCreateInfo {
      flags: PipelineCreateFlags::ALLOW_DERIVATIVES,
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
      render_pass,
      pipeline: pipelines[0],
    }
  }

  pub(crate) fn drop(&self) {
    unsafe {
      self.device.destroy_pipeline(self.pipeline, None);
      self.device.destroy_render_pass(self.render_pass, None);
      self.device.destroy_pipeline_layout(self.layout, None);
    }
  }
}
