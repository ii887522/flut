use ash::vk;

#[repr(C, align(8))]
pub(crate) struct RectPushConst {
  rect_buf_addr: vk::DeviceAddress,
  cam_size: (f32, f32),
}

impl RectPushConst {
  pub(crate) const fn new(rect_buf_addr: vk::DeviceAddress, cam_size: (u32, u32)) -> Self {
    Self {
      rect_buf_addr,
      cam_size: (cam_size.0 as _, cam_size.1 as _),
    }
  }
}
