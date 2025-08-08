use crate::models::Rect;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Glass {
  pub(crate) size: (f32, f32),
  pub(crate) alpha: f32,
  pub(crate) drawable_id: u32,
}

impl From<Glass> for Rect {
  fn from(glass: Glass) -> Self {
    Self::new()
      .size(glass.size)
      .color((0, 0, 0, (glass.alpha * 255.0) as _))
      .call()
  }
}
