#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Rect {
  position: (f32, f32),
  size: (f32, f32),
  color: (f32, f32, f32, f32),
  tex_position: (f32, f32),
  pad: (f32, f32),
}

impl Rect {
  pub const fn new(position: (f32, f32), size: (f32, f32), color: (u8, u8, u8)) -> Self {
    Self {
      position,
      size,
      color: (
        color.0 as f32 / 255.0,
        color.1 as f32 / 255.0,
        color.2 as f32 / 255.0,
        1.0,
      ),
      tex_position: (-size.0, -size.1),
      pad: (0.0, 0.0),
    }
  }
}
