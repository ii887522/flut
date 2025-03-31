use ash::{
  Device,
  vk::{PipelineShaderStageCreateInfo, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags},
};
use std::{ffi::CString, rc::Rc};

pub(crate) struct RectFragShader<'a> {
  device: Rc<Device>,
  shader: ShaderModule,
  _entry_point_name: CString,
  pub(crate) shader_stage_create_info: PipelineShaderStageCreateInfo<'a>,
}

impl RectFragShader<'_> {
  pub(crate) fn new(device: Rc<Device>) -> Self {
    const SHADER_CODE: &[u8] = include_bytes!("../../target/shaders/rect.frag.spv");

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
      stage: ShaderStageFlags::FRAGMENT,
      module: shader,
      p_name: shader_entry_point_name.as_ptr(),
      ..Default::default()
    };

    Self {
      device,
      shader,
      _entry_point_name: shader_entry_point_name,
      shader_stage_create_info,
    }
  }
}

impl Drop for RectFragShader<'_> {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_shader_module(self.shader, None);
    }
  }
}
