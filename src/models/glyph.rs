use std::cmp::Ordering;
use voracious_radix_sort::Radixable;

#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct Glyph {
  pub position: (f32, f32, f32),
  pub color: u32,
  pub size: (f32, f32),
  pub atlas_position: (f32, f32),
}

impl PartialEq for Glyph {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.position.2 == other.position.2
  }
}

impl PartialOrd for Glyph {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.position.2.partial_cmp(&other.position.2)
  }
}

impl Radixable<f32> for Glyph {
  type Key = f32;

  #[inline]
  fn key(&self) -> Self::Key {
    self.position.2
  }
}

impl Glyph {
  #[inline]
  pub(crate) const fn get_vertex_count() -> usize {
    6
  }
}
