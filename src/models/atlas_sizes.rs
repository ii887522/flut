#[derive(Clone, Copy)]
pub struct AtlasSizes {
  pub glyph_atlas_size: (u32, u32),
}

impl Default for AtlasSizes {
  #[inline]
  fn default() -> Self {
    Self {
      glyph_atlas_size: (1024, 1024),
    }
  }
}
