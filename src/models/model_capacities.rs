use crate::pipelines::glyph_pipeline::Glyph;
use std::mem;

#[derive(Clone, Copy)]
pub struct ModelCapacities {
  pub glyph_capacity: usize,
}

impl Default for ModelCapacities {
  #[inline]
  fn default() -> Self {
    Self {
      glyph_capacity: 1024,
    }
  }
}

impl ModelCapacities {
  #[inline]
  pub(crate) const fn calc_total_size(&self) -> usize {
    self.glyph_capacity * mem::size_of::<Glyph>()
  }
}
