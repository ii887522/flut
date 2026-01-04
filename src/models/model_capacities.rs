use crate::pipelines::{glyph_pipeline::Glyph, round_rect_pipeline::RoundRect};
use std::mem;

#[derive(Clone)]
pub struct ModelCapacities {
  pub glyph_capacities: Box<[usize]>,
  pub round_rect_capacities: Box<[usize]>,
}

impl Default for ModelCapacities {
  #[inline]
  fn default() -> Self {
    Self {
      glyph_capacities: Box::new([1024]),
      round_rect_capacities: Box::new([1024]),
    }
  }
}

impl ModelCapacities {
  #[inline]
  pub(crate) fn calc_total_size(&self) -> usize {
    self.glyph_capacities.iter().sum::<usize>() * mem::size_of::<Glyph>()
      + self.round_rect_capacities.iter().sum::<usize>() * mem::size_of::<RoundRect>()
  }

  #[inline]
  pub(crate) fn calc_total_capacity(&self) -> usize {
    self.glyph_capacities.iter().sum::<usize>() + self.round_rect_capacities.iter().sum::<usize>()
  }
}
