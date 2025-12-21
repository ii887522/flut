use crate::models::Align;
use std::borrow::Cow;

#[derive(Clone)]
pub struct Text {
  pub position: (f32, f32),
  pub color: (f32, f32, f32, f32),
  pub font_size: u16,
  pub align: Align,
  pub text: Cow<'static, str>,
}
