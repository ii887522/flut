use crate::{Transition, models::Rect};

#[derive(Clone, Copy)]
pub struct Glass {
  pub size: (f32, f32),
  pub alpha: Transition,
  pub drawable_id: u16,
}

impl From<Glass> for Rect {
  fn from(glass: Glass) -> Self {
    Self::new(
      (0.0, 0.0),
      glass.size,
      (0, 0, 0, glass.alpha.get_value() as _),
    )
  }
}
