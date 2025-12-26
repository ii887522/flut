#[derive(Clone, Copy)]
pub struct RoundRect {
  pub position: (f32, f32),
  pub size: (f32, f32),
  pub color: (f32, f32, f32, f32),
  pub radius: f32,
}
