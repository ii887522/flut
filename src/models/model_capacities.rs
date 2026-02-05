use crate::models::round_rect::RoundRect;
use std::mem;

#[derive(Clone, Copy)]
pub struct ModelCapacities {
  pub round_rect_capacity: usize,
}

impl Default for ModelCapacities {
  #[inline]
  fn default() -> Self {
    Self {
      round_rect_capacity: 1024,
    }
  }
}

impl ModelCapacities {
  #[inline]
  pub(crate) const fn calc_bytes(self) -> usize {
    self.round_rect_capacity * mem::size_of::<RoundRect>()
  }
}
