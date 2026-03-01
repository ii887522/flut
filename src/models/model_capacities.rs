use crate::models::{glyph::Glyph, round_rect::RoundRect};
use std::mem;

#[derive(Clone, Copy)]
pub struct ModelCapacities {
  pub round_rect_capacity: usize,
  pub clipped_round_rect_capacity: usize,
  pub glyph_capacity: usize,
  pub clipped_glyph_capacity: usize,
}

impl Default for ModelCapacities {
  #[inline]
  fn default() -> Self {
    Self {
      round_rect_capacity: 1024,
      clipped_round_rect_capacity: 32,
      glyph_capacity: 1024,
      clipped_glyph_capacity: 32,
    }
  }
}

impl ModelCapacities {
  #[inline]
  pub(crate) const fn calc_bytes(self) -> usize {
    (self.round_rect_capacity + self.clipped_round_rect_capacity) * mem::size_of::<RoundRect>()
      + (self.glyph_capacity + self.clipped_glyph_capacity) * mem::size_of::<Glyph>()
  }
}
