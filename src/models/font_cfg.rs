use skia_safe::font_style::{Slant, Weight, Width};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontCfg {
  pub font_family: &'static str,
  pub font_weight: Weight,
  pub font_width: Width,
  pub font_slant: Slant,
  pub font_size: u8,
}

impl Default for FontCfg {
  fn default() -> Self {
    Self {
      font_family: "Arial",
      font_weight: Weight::NORMAL,
      font_width: Width::NORMAL,
      font_slant: Slant::Upright,
      font_size: 12,
    }
  }
}

impl Hash for FontCfg {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.font_family.hash(state);
    self.font_weight.hash(state);
    self.font_width.hash(state);
    self.font_slant.hash(state);
    self.font_size.hash(state);
  }
}
