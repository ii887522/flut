use crate::models::font_key::FontKey;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq)]
pub struct GlyphKey {
  pub font_key: FontKey,
  pub ch: char,
  pub font_size: f32,
}

impl Hash for GlyphKey {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    let Self {
      ref font_key,
      ch,
      font_size,
    } = *self;

    font_key.hash(state);
    ch.hash(state);
    font_size.to_bits().hash(state);
  }
}

impl Eq for GlyphKey {}
