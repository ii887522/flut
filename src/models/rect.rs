#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct Rect {
  position: (f32, f32),
  size: (f32, f32),
  color: u32,
  pad: f32,
}
