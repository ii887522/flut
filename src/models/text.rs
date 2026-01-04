use crate::models::{Align, Icon};
use std::borrow::Cow;

#[derive(Clone)]
pub struct Text {
  pub position: (f32, f32, u8),
  pub font_size: f32,
  pub color: (u8, u8, u8, u8),
  pub align: Align,
  pub text: Cow<'static, str>,
}

impl From<Icon> for Text {
  fn from(icon: Icon) -> Self {
    Text {
      position: icon.position,
      font_size: icon.font_size,
      color: icon.color,
      align: Align::Left,
      text: String::from_utf16_lossy(&[icon.codepoint]).into(),
    }
  }
}
