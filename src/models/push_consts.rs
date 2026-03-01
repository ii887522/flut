use ash::vk;

#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct PushConsts {
  pub round_rect_buffer: vk::DeviceAddress,
  pub glyph_buffer: vk::DeviceAddress,
  pub cam_size: (f32, f32),
  pub glyph_atlas_size: (f32, f32),
}
