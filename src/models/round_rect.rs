#[derive(Clone, Copy)]
pub struct RoundRect {
  pub(super) position: (f32, f32, f32),
  pub(super) size: (f32, f32),
  pub(super) color: (u8, u8, u8, u8),
  pub(super) border_radius: f32,
}

impl RoundRect {
  pub const fn new(
    position: (f32, f32, f32),
    size: (f32, f32),
    color: (u8, u8, u8, u8),
    border_radius: f32,
  ) -> Self {
    Self {
      position,
      size,
      color,
      border_radius,
    }
  }
}
