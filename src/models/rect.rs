#[repr(C, align(8))]
pub(crate) struct Rect {
  position: (f32, f32),
  size: (f32, f32),
  color: u32,
  pad: f32,
}
