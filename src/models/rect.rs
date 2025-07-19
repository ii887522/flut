#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct Rect {
  position: (f32, f32),
  size: (f32, f32),
  color: u32,
  pad: f32,
}

impl Rect {
  pub const fn new(position: (f32, f32), size: (f32, f32), color: (u8, u8, u8)) -> Self {
    Self {
      position,
      size,
      color: ((color.0 as u32) << 24) | ((color.1 as u32) << 16) | ((color.2 as u32) << 8),
      pad: 0.0,
    }
  }
}
