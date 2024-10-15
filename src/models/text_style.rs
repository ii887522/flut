use crate::helpers::consts;
use skia_safe::{Color, FontStyle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
  pub font_family: &'static str,
  pub font_style: FontStyle,
  pub font_size: f32,
  pub color: Color,
}

impl Default for TextStyle {
  fn default() -> Self {
    Self {
      font_family: consts::DEFAULT_FONT_FAMILY,
      font_style: FontStyle::default(),
      font_size: 12.0,
      color: Color::BLACK,
    }
  }
}
