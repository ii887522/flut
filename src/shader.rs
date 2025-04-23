use ash::{
  Device,
  vk::{PipelineShaderStageCreateInfo, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags},
};
use std::{ffi::CString, sync::Arc};

pub(super) struct Shader<'a> {
  device: Arc<Device>,
  shader: ShaderModule,
  _entry_point_name: CString,
  pub(super) shader_stage_create_info: PipelineShaderStageCreateInfo<'a>,
}

impl Shader<'_> {
  pub(crate) fn new(device: Arc<Device>, stage: ShaderStageFlags, code: &[u8]) -> Self {
    let shader_create_info = ShaderModuleCreateInfo {
      code_size: code.len(),
      p_code: code.as_ptr() as *const _,
      ..Default::default()
    };

    let shader = unsafe {
      device
        .create_shader_module(&shader_create_info, None)
        .unwrap()
    };

    let shader_entry_point_name = CString::new("main").unwrap();

    let shader_stage_create_info = PipelineShaderStageCreateInfo {
      stage,
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

impl Drop for Shader<'_> {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_shader_module(self.shader, None);
    }
  }
}
