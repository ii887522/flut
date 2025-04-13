use ash::{
  Device,
  vk::{
    self, BorderColor, CompareOp, Filter, PipelineShaderStageCreateInfo, Sampler,
    SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode, ShaderModule, ShaderModuleCreateInfo,
    ShaderStageFlags,
  },
};
use std::{ffi::CString, rc::Rc};

pub(crate) struct RectFragShader<'a> {
  device: Rc<Device>,
  shader: ShaderModule,
  _entry_point_name: CString,
  pub(crate) sampler: Sampler,
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

    let sampler_create_info = SamplerCreateInfo {
      mag_filter: Filter::NEAREST,
      min_filter: Filter::NEAREST,
      mipmap_mode: SamplerMipmapMode::NEAREST,
      address_mode_u: SamplerAddressMode::CLAMP_TO_BORDER,
      address_mode_v: SamplerAddressMode::CLAMP_TO_BORDER,
      address_mode_w: SamplerAddressMode::CLAMP_TO_BORDER,
      mip_lod_bias: 0.0,
      anisotropy_enable: vk::FALSE,
      compare_enable: vk::FALSE,
      border_color: BorderColor::INT_OPAQUE_WHITE,
      unnormalized_coordinates: vk::TRUE,
      ..Default::default()
    };

    let sampler = unsafe { device.create_sampler(&sampler_create_info, None).unwrap() };

    Self {
      device,
      shader,
      _entry_point_name: shader_entry_point_name,
      sampler,
      shader_stage_create_info,
    }
  }
}

impl Drop for RectFragShader<'_> {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_sampler(self.sampler, None);
      self.device.destroy_shader_module(self.shader, None);
    }
  }
}
