use crate::models::{align::Align, font_key::FontKey};
use std::borrow::Cow;

#[derive(Clone)]
pub struct Text {
  pub position: (f32, f32, f32),
  pub color: u32,
  pub font_size: f32,
  pub font_key: FontKey,
  pub align: Align,
  pub text: Cow<'static, str>,
}
