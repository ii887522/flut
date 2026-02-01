#[repr(C, align(16))]
pub struct RoundRect {
  pub position: (f32, f32, f32),
  pub radius: f32,
  pub size: (f32, f32),
  pub color: u32,
}
