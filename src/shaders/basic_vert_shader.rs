use ash::{
  Device,
  vk::{
    Format, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, ShaderModule,
    ShaderModuleCreateInfo, ShaderStageFlags, VertexInputAttributeDescription,
    VertexInputBindingDescription, VertexInputRate,
  },
};
use std::{ffi::CString, mem, rc::Rc};

#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub(crate) struct Instance {
  position: (f32, f32),
  color: (f32, f32, f32),
}

impl Instance {
  pub(crate) const fn new(position: (f32, f32), color: (u8, u8, u8)) -> Self {
    Self {
      position,
      color: (
        color.0 as f32 / 255.0,
        color.1 as f32 / 255.0,
        color.2 as f32 / 255.0,
      ),
    }
  }
}

#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub(crate) struct Vertex {
  pub(crate) position: (f32, f32),
}

pub(crate) struct BasicVertShader<'a> {
  device: Rc<Device>,
  shader: ShaderModule,
  _entry_point_name: CString,
  pub(crate) shader_stage_create_info: PipelineShaderStageCreateInfo<'a>,
  _binding_descs: Vec<VertexInputBindingDescription>,
  _attr_descs: Vec<VertexInputAttributeDescription>,
  pub(crate) vert_input_stage_create_info: PipelineVertexInputStateCreateInfo<'a>,
}

impl<'a> BasicVertShader<'a> {
  pub(crate) fn new(device: Rc<Device>) -> Self {
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

    let vert_binding_descs = vec![
      VertexInputBindingDescription {
        binding: 0,
        stride: size_of::<Vertex>() as _,
        input_rate: VertexInputRate::VERTEX,
      },
      VertexInputBindingDescription {
        binding: 1,
        stride: size_of::<Instance>() as _,
        input_rate: VertexInputRate::INSTANCE,
      },
    ];

    let vert_attr_descs = vec![
      VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: Format::R32G32_SFLOAT,
        offset: mem::offset_of!(Vertex, position) as _,
      },
      VertexInputAttributeDescription {
        location: 1,
        binding: 1,
        format: Format::R32G32_SFLOAT,
        offset: mem::offset_of!(Instance, position) as _,
      },
      VertexInputAttributeDescription {
        location: 2,
        binding: 1,
        format: Format::R32G32B32_SFLOAT,
        offset: mem::offset_of!(Instance, color) as _,
      },
    ];

    let vert_input_stage_create_info = PipelineVertexInputStateCreateInfo {
      vertex_binding_description_count: vert_binding_descs.len() as _,
      p_vertex_binding_descriptions: vert_binding_descs.as_ptr(),
      vertex_attribute_description_count: vert_attr_descs.len() as _,
      p_vertex_attribute_descriptions: vert_attr_descs.as_ptr(),
      ..Default::default()
    };

    Self {
      device,
      shader,
      _entry_point_name: shader_entry_point_name,
      shader_stage_create_info,
      _binding_descs: vert_binding_descs,
      _attr_descs: vert_attr_descs,
      vert_input_stage_create_info,
    }
  }

  pub(crate) fn drop(&self) {
    unsafe {
      self.device.destroy_shader_module(self.shader, None);
    }
  }
}
