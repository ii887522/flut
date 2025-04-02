#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct Rect {
  pub(crate) position: (f32, f32),
  pub(crate) color: (f32, f32, f32),
}

impl Rect {
  pub const fn new(position: (f32, f32), color: (u8, u8, u8)) -> Self {
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
