use crate::{Transition, models::Rect};

#[derive(Clone, Copy)]
pub(super) struct Glass {
  pub(super) size: (f32, f32),
  pub(super) alpha: Transition,
  pub(super) drawable_id: u16,
}

impl Glass {
  pub(super) fn update(&mut self, dt: f32) -> bool {
    self.alpha.update(dt)
  }
}

impl From<Glass> for Rect {
  fn from(glass: Glass) -> Self {
    Self::new(
      (0.0, 0.0, 1.0),
      glass.size,
      (0, 0, 0, glass.alpha.get_value() as _),
    )
  }
}
