use crate::models::Rect;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Container {
  pub(crate) position: (f32, f32),
  pub(crate) size: (f32, f32),
  pub(crate) color: (u8, u8, u8, u8),
  pub(crate) drawable_id: u32,
}

impl From<Container> for Rect {
  fn from(container: Container) -> Self {
    Self::new()
      .position(container.position)
      .size(container.size)
      .color(container.color)
      .call()
  }
}
