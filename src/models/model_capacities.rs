use crate::pipelines::{glyph_pipeline::Glyph, rect_pipeline::Rect};
use std::mem;

#[derive(Clone, Copy)]
pub struct ModelCapacities {
  pub rect_capacity: usize,
  pub glyph_capacity: usize,
}

impl Default for ModelCapacities {
  #[inline]
  fn default() -> Self {
    Self {
      rect_capacity: 1024,
      glyph_capacity: 1024,
    }
  }
}

impl ModelCapacities {
  #[inline]
  pub(crate) const fn calc_total_size(&self) -> usize {
    self.rect_capacity * mem::size_of::<Rect>() + self.glyph_capacity * mem::size_of::<Glyph>()
  }
}
