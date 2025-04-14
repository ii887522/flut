use ash::{
  Device,
  vk::{
    PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, ShaderModule,
    ShaderModuleCreateInfo, ShaderStageFlags,
  },
};
use std::{ffi::CString, rc::Rc};

pub(crate) struct GlyphVertShader<'a> {
  device: Rc<Device>,
  shader: ShaderModule,
  _entry_point_name: CString,
  pub(crate) shader_stage_create_info: PipelineShaderStageCreateInfo<'a>,
  pub(crate) vert_input_stage_create_info: PipelineVertexInputStateCreateInfo<'a>,
}

impl GlyphVertShader<'_> {
  pub(crate) fn new(device: Rc<Device>) -> Self {
    const SHADER_CODE: &[u8] = include_bytes!("../../target/shaders/text.vert.spv");

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

    let vert_input_stage_create_info = PipelineVertexInputStateCreateInfo::default();

    Self {
      device,
      shader,
      _entry_point_name: shader_entry_point_name,
      shader_stage_create_info,
      vert_input_stage_create_info,
    }
  }
}

impl Drop for GlyphVertShader<'_> {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_shader_module(self.shader, None);
    }
  }
}
