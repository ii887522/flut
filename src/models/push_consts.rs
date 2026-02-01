use ash::vk;

#[repr(C, align(8))]
pub struct PushConsts {
  pub round_rect_buffer: vk::DeviceAddress,
  pub cam_size: (f32, f32),
}
