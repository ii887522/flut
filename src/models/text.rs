use crate::models::Anchor;
use optarg2chain::optarg_impl;
use std::borrow::Cow;

#[derive(Clone)]
pub struct Text {
  pub(crate) position: (f32, f32),
  pub(crate) font_size: u16,
  pub(crate) text: Cow<'static, str>,
  pub(crate) color: (u8, u8, u8, u8),
  pub(crate) anchor: Anchor,
}

#[optarg_impl]
impl Text {
  #[optarg_method(TextNewBuilder, call)]
  pub fn new(
    #[optarg_default] position: (f32, f32),
    #[optarg(16)] font_size: u16,
    #[optarg_default] text: Cow<'static, str>,
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    #[optarg_default] anchor: Anchor,
  ) -> Self {
    Self {
      position,
      font_size,
      text,
      color,
      anchor,
    }
  }
}
