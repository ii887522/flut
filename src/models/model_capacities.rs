use crate::pipelines::{glyph_pipeline::Glyph, round_rect_pipeline::RoundRect};
use std::mem;

#[derive(Clone, Copy)]
pub struct ModelCapacities {
  pub glyph_capacity: usize,
  pub round_rect_capacity: usize,
}

impl Default for ModelCapacities {
  #[inline]
  fn default() -> Self {
    Self {
      glyph_capacity: 1024,
      round_rect_capacity: 1024,
    }
  }
}

impl ModelCapacities {
  #[inline]
  pub(crate) const fn calc_total_size(&self) -> usize {
    self.glyph_capacity * mem::size_of::<Glyph>()
      + self.round_rect_capacity * mem::size_of::<RoundRect>()
  }
}
