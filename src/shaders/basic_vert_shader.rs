use ash::{
  Device,
  vk::{
    Format, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, ShaderModule,
    ShaderModuleCreateInfo, ShaderStageFlags, VertexInputAttributeDescription,
    VertexInputBindingDescription, VertexInputRate,
  },
};
use std::{ffi::CString, mem};

pub(crate) struct Vertex {
  position: (f32, f32),
}

pub(crate) struct BasicVertShader<'a> {
  device: &'a Device,
  shader: ShaderModule,
  _entry_point_name: CString,
  pub(crate) shader_stage_create_info: PipelineShaderStageCreateInfo<'a>,
  _binding_desc: Box<VertexInputBindingDescription>,
  _position_desc: Box<VertexInputAttributeDescription>,
  pub(crate) vert_input_stage_create_info: PipelineVertexInputStateCreateInfo<'a>,
}

impl<'a> BasicVertShader<'a> {
  pub(crate) fn new(device: &'a Device) -> Self {
    const SHADER_CODE: &[u8] = include_bytes!("../../target/shaders/basic.vert.spv");

    let shader_create_info = ShaderModuleCreateInfo {
      code_size: SHADER_CODE.len(),
      p_code: SHADER_CODE.as_ptr() as *const _,
      ..Default::default()
    };

    let shader = unsafe {
      device
        .create_shader_module(&shader_create_info, None)
        .unwrap()
    };

    let shader_entry_point_name = CString::new("main").unwrap();

    let shader_stage_create_info = PipelineShaderStageCreateInfo {
      stage: ShaderStageFlags::VERTEX,
      module: shader,
      p_name: shader_entry_point_name.as_ptr(),
      ..Default::default()
    };

    let binding_desc = Box::new(VertexInputBindingDescription {
      binding: 0,
      stride: size_of::<Vertex>() as _,
      input_rate: VertexInputRate::VERTEX,
    });

    let position_desc = Box::new(VertexInputAttributeDescription {
      location: 0,
      binding: 0,
      format: Format::R32G32_SFLOAT,
      offset: mem::offset_of!(Vertex, position) as _,
    });

    let vert_input_stage_create_info = PipelineVertexInputStateCreateInfo {
      vertex_binding_description_count: 1,
      p_vertex_binding_descriptions: &*binding_desc,
      vertex_attribute_description_count: 1,
      p_vertex_attribute_descriptions: &*position_desc,
      ..Default::default()
    };

    Self {
      device,
      shader,
      _entry_point_name: shader_entry_point_name,
      shader_stage_create_info,
      _binding_desc: binding_desc,
      _position_desc: position_desc,
      vert_input_stage_create_info,
    }
  }
}

impl Drop for BasicVertShader<'_> {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_shader_module(self.shader, None);
    }
  }
}
