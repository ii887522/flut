use super::Device;
use ash::vk::{self, Handle};
use std::{ffi::CString, rc::Rc};

pub(super) struct Creating;

pub(super) struct Created;

pub(super) struct GraphicsPipeline<State> {
  device: Rc<Device>,
  cache: vk::PipelineCache,
  vert_shader_module: vk::ShaderModule,
  frag_shader_module: vk::ShaderModule,
  layout: vk::PipelineLayout,
  pipeline: vk::Pipeline,
  _state: State,
}

impl GraphicsPipeline<Creating> {
  pub(super) fn new(
    vk_device: Rc<Device>,
    vert_shader_code: &[u8],
    frag_shader_code: &[u8],
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
    push_const_ranges: &[vk::PushConstantRange],
  ) -> Self {
    let device = vk_device.get();

    let cache_create_info = vk::PipelineCacheCreateInfo {
      flags: if vk_device.pipeline_creation_cache_control() {
        vk::PipelineCacheCreateFlags::EXTERNALLY_SYNCHRONIZED
      } else {
        vk::PipelineCacheCreateFlags::empty()
      },
      ..Default::default()
    };

    let cache = unsafe {
      device
        .create_pipeline_cache(&cache_create_info, None)
        .unwrap()
    };

    let vert_shader_module_create_info = vk::ShaderModuleCreateInfo {
      code_size: vert_shader_code.len(),
      p_code: vert_shader_code as *const _ as *const _,
      ..Default::default()
    };

    let frag_shader_module_create_info = vk::ShaderModuleCreateInfo {
      code_size: frag_shader_code.len(),
      p_code: frag_shader_code as *const _ as *const _,
      ..Default::default()
    };

    let vert_shader_module = unsafe {
      device
        .create_shader_module(&vert_shader_module_create_info, None)
        .unwrap()
    };

    let frag_shader_module = unsafe {
      device
        .create_shader_module(&frag_shader_module_create_info, None)
        .unwrap()
    };

    let layout_create_info = vk::PipelineLayoutCreateInfo {
      set_layout_count: descriptor_set_layouts.len() as _,
      p_set_layouts: descriptor_set_layouts.as_ptr(),
      push_constant_range_count: push_const_ranges.len() as _,
      p_push_constant_ranges: push_const_ranges.as_ptr(),
      ..Default::default()
    };

    let layout = unsafe {
      device
        .create_pipeline_layout(&layout_create_info, None)
        .unwrap()
    };

    Self {
      device: vk_device,
      cache,
      vert_shader_module,
      frag_shader_module,
      layout,
      pipeline: vk::Pipeline::null(),
      _state: Creating,
    }
  }

  pub(super) fn finish(
    self,
    render_pass: vk::RenderPass,
    swapchain_image_extent: vk::Extent2D,
  ) -> GraphicsPipeline<Created> {
    let device = self.device.get();
    let main_name = CString::new("main").unwrap();

    let flags = vk::PipelineCreateFlags::ALLOW_DERIVATIVES;

    let flags = if self.pipeline.is_null() {
      flags
    } else {
      flags | vk::PipelineCreateFlags::DERIVATIVE
    };

    let shader_stage_create_infos = [
      vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::VERTEX,
        module: self.vert_shader_module,
        p_name: main_name.as_ptr(),
        ..Default::default()
      },
      vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: self.frag_shader_module,
        p_name: main_name.as_ptr(),
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
      cull_mode: vk::CullModeFlags::NONE,
      front_face: vk::FrontFace::CLOCKWISE,
      line_width: 1.0,
      ..Default::default()
    };

    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
      rasterization_samples: vk::SampleCountFlags::TYPE_1,
      ..Default::default()
    };

    let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::default();

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
      blend_enable: vk::TRUE,
      src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
      dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
      color_blend_op: vk::BlendOp::ADD,
      src_alpha_blend_factor: vk::BlendFactor::ONE,
      dst_alpha_blend_factor: vk::BlendFactor::ZERO,
      alpha_blend_op: vk::BlendOp::ADD,
      color_write_mask: vk::ColorComponentFlags::RGBA,
    }];

    let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo {
      attachment_count: color_blend_attachment_states.len() as _,
      p_attachments: color_blend_attachment_states.as_ptr(),
      ..Default::default()
    };

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
      flags,
      stage_count: shader_stage_create_infos.len() as _,
      p_stages: shader_stage_create_infos.as_ptr(),
      p_vertex_input_state: &vertex_input_state_create_info,
      p_input_assembly_state: &input_assembly_state_create_info,
      p_viewport_state: &viewport_state_create_info,
      p_rasterization_state: &rasterization_state_create_info,
      p_multisample_state: &multisample_state_create_info,
      p_depth_stencil_state: &depth_stencil_state_create_info,
      p_color_blend_state: &color_blend_state_create_info,
      layout: self.layout,
      render_pass,
      subpass: 0,
      base_pipeline_handle: self.pipeline,
      base_pipeline_index: -1,
      ..Default::default()
    };

    let pipeline = unsafe {
      device
        .create_graphics_pipelines(self.cache, &[pipeline_create_info], None)
        .unwrap()[0]
    };

    unsafe {
      device.destroy_pipeline(self.pipeline, None);
    }

    GraphicsPipeline {
      device: self.device,
      cache: self.cache,
      vert_shader_module: self.vert_shader_module,
      frag_shader_module: self.frag_shader_module,
      layout: self.layout,
      pipeline,
      _state: Created,
    }
  }
}

impl GraphicsPipeline<Created> {
  pub(super) const fn get(&self) -> vk::Pipeline {
    self.pipeline
  }

  pub(super) fn on_swapchain_suboptimal(self) -> GraphicsPipeline<Creating> {
    GraphicsPipeline {
      device: self.device,
      cache: self.cache,
      vert_shader_module: self.vert_shader_module,
      frag_shader_module: self.frag_shader_module,
      layout: self.layout,
      pipeline: self.pipeline,
      _state: Creating,
    }
  }
}

impl<State> GraphicsPipeline<State> {
  pub(super) const fn get_layout(&self) -> vk::PipelineLayout {
    self.layout
  }

  pub(super) fn drop(self) {
    let device = self.device.get();

    unsafe {
      device.destroy_pipeline(self.pipeline, None);
      device.destroy_pipeline_layout(self.layout, None);
      device.destroy_shader_module(self.frag_shader_module, None);
      device.destroy_shader_module(self.vert_shader_module, None);
      device.destroy_pipeline_cache(self.cache, None);
    }
  }
}
