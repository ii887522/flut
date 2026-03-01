use crate::models::align::Align;
use font_kit::{family_name::FamilyName, properties::Properties};
use std::borrow::Cow;

#[derive(Clone)]
pub struct Text {
  pub position: (f32, f32, f32),
  pub color: u32,
  pub font_size: f32,
  pub font_family: Cow<'static, [FamilyName]>,
  pub font_props: Properties,
  pub align: Align,
  pub text: Cow<'static, str>,
}
