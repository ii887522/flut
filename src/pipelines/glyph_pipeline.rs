use crate::{
  models,
  pipelines::{CreatedPipeline, CreatingPipeline, Model},
  utils,
};
use ash::vk::{self, Handle};
use std::{ffi::CString, mem};

const VERT_SHADER_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/spv/glyph.vert.spv"));
const FRAG_SHADER_CODE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/spv/glyph.frag.spv"));

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Glyph {
  pub(crate) position: (f32, f32),
  pub(crate) size: (f32, f32),
  pub(crate) atlas_position: (f32, f32),
  pub(crate) atlas_size: (f32, f32),
  pub(crate) color: u32,
  pub(crate) pad: f32,
}

impl Model for Glyph {
  type PushConsts = GlyphPushConsts;
  type CreatingPipeline = GlyphPipeline<Creating>;
  type CreatedPipeline = GlyphPipeline<Created>;

  #[inline]
  fn get_name() -> &'static str {
    "glyph"
  }

  #[inline]
  fn get_vertex_count() -> usize {
    6
  }
}

impl From<models::Rect> for Glyph {
  fn from(rect: models::Rect) -> Self {
    Self {
      position: (rect.position.0, rect.position.1),
      size: rect.size,
      atlas_position: (-1.0, -1.0),
      atlas_size: (0.0, 0.0),
      color: utils::pack_color(rect.color),
      pad: 0.0,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct GlyphPushConsts {
  pub(crate) glyph_buffer: vk::DeviceAddress,
  pub(crate) cam_position: (f32, f32),
  pub(crate) cam_size: (f32, f32),
  pub(crate) atlas_size: (f32, f32),
}

pub(crate) struct Creating;
pub(crate) struct Created;

pub(crate) struct GlyphPipeline<State> {
  vert_shader_module: vk::ShaderModule,
  frag_shader_module: vk::ShaderModule,
  sampler: vk::Sampler,
  descriptor_set_layout: vk::DescriptorSetLayout,
  layout: vk::PipelineLayout,
  pipeline: vk::Pipeline,
  _state: State,
}

impl CreatingPipeline for GlyphPipeline<Creating> {
  type Model = Glyph;

  fn new(device: &ash::Device) -> Self {
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

    let sampler_create_info = vk::SamplerCreateInfo {
      mag_filter: vk::Filter::LINEAR,
      min_filter: vk::Filter::LINEAR,
      mipmap_mode: vk::SamplerMipmapMode::NEAREST,
      address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
      address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
      address_mode_w: vk::SamplerAddressMode::CLAMP_TO_BORDER,
      min_lod: 0.0,
      max_lod: vk::LOD_CLAMP_NONE,
      border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
      ..Default::default()
    };

    let sampler = unsafe { device.create_sampler(&sampler_create_info, None).unwrap() };

    let descriptor_set_layout_bindings = [vk::DescriptorSetLayoutBinding {
      binding: 0,
      descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
      descriptor_count: 1,
      stage_flags: vk::ShaderStageFlags::FRAGMENT,
      p_immutable_samplers: &sampler,
      ..Default::default()
    }];

    let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
      binding_count: descriptor_set_layout_bindings.len() as _,
      p_bindings: descriptor_set_layout_bindings.as_ptr(),
      ..Default::default()
    };

    let descriptor_set_layout = unsafe {
      device
        .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
        .unwrap()
    };

    let push_const_ranges = [vk::PushConstantRange {
      stage_flags: vk::ShaderStageFlags::VERTEX,
      offset: 0,
      size: mem::size_of::<GlyphPushConsts>() as _,
    }];

    let descriptor_set_layouts = [descriptor_set_layout];

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
      vert_shader_module,
      frag_shader_module,
      sampler,
      descriptor_set_layout,
      layout,
      pipeline: vk::Pipeline::null(),
      _state: Creating,
    }
  }

  fn finish(
    self,
    device: &ash::Device,
    render_pass: vk::RenderPass,
    cache: vk::PipelineCache,
    swapchain_image_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
  ) -> GlyphPipeline<Created> {
    let main_fn_name = CString::new("main").unwrap();

    let shader_stage_create_infos = [
      vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::VERTEX,
        module: self.vert_shader_module,
        p_name: main_fn_name.as_ptr(),
        ..Default::default()
      },
      vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: self.frag_shader_module,
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
      rasterization_samples: msaa_samples,
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

    let flags = if self.pipeline.is_null() {
      vk::PipelineCreateFlags::ALLOW_DERIVATIVES
    } else {
      vk::PipelineCreateFlags::ALLOW_DERIVATIVES | vk::PipelineCreateFlags::DERIVATIVE
    };

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
      flags,
      stage_count: shader_stage_create_infos.len() as _,
      p_stages: shader_stage_create_infos.as_ptr(),
      p_vertex_input_state: &vertex_input_state_create_info,
      p_input_assembly_state: &input_assembly_state_create_info,
      p_viewport_state: &viewport_state_create_info,
      p_rasterization_state: &rasterization_state_create_info,
      p_multisample_state: &multisample_state,
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
        .create_graphics_pipelines(cache, &[pipeline_create_info], None)
        .unwrap()[0]
    };

    unsafe {
      device.destroy_pipeline(self.pipeline, None);
    }

    GlyphPipeline {
      vert_shader_module: self.vert_shader_module,
      frag_shader_module: self.frag_shader_module,
      sampler: self.sampler,
      descriptor_set_layout: self.descriptor_set_layout,
      layout: self.layout,
      pipeline,
      _state: Created,
    }
  }

  #[inline]
  fn get_descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
    self.descriptor_set_layout
  }

  fn drop(self, device: &ash::Device) {
    unsafe {
      device.destroy_pipeline(self.pipeline, None);
      device.destroy_pipeline_layout(self.layout, None);
      device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
      device.destroy_sampler(self.sampler, None);
      device.destroy_shader_module(self.frag_shader_module, None);
      device.destroy_shader_module(self.vert_shader_module, None);
    }
  }
}

impl CreatedPipeline for GlyphPipeline<Created> {
  type Model = Glyph;

  #[inline]
  fn get_pipeline(&self) -> vk::Pipeline {
    self.pipeline
  }

  #[inline]
  fn get_pipeline_layout(&self) -> vk::PipelineLayout {
    self.layout
  }

  fn on_swapchain_suboptimal(self) -> GlyphPipeline<Creating> {
    GlyphPipeline {
      vert_shader_module: self.vert_shader_module,
      frag_shader_module: self.frag_shader_module,
      sampler: self.sampler,
      descriptor_set_layout: self.descriptor_set_layout,
      layout: self.layout,
      pipeline: self.pipeline,
      _state: Creating,
    }
  }

  fn drop(self, device: &ash::Device) {
    unsafe {
      device.destroy_pipeline(self.pipeline, None);
      device.destroy_pipeline_layout(self.layout, None);
      device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
      device.destroy_sampler(self.sampler, None);
      device.destroy_shader_module(self.frag_shader_module, None);
      device.destroy_shader_module(self.vert_shader_module, None);
    }
  }
}
