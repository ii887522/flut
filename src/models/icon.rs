use crate::models::font_key::FontKey;

#[derive(Clone)]
pub struct Icon {
  pub position: (f32, f32, f32),
  pub color: u32,
  pub font_size: f32,
  pub font_key: FontKey,
  pub codepoint: u16,
}
