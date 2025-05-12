use super::{Glyph, IconName};
use crate::atlases::IconAtlas;

#[derive(Clone, Copy)]
pub struct Icon {
  position: (f32, f32),
  color: (u8, u8, u8, u8),
  code_point: u16,
}

impl Icon {
  pub const fn new(position: (f32, f32), color: (u8, u8, u8, u8), name: IconName) -> Self {
    Self {
      position,
      color,
      code_point: name as _,
    }
  }

  pub(crate) fn into_glyph(self, icon_atlas: &mut IconAtlas<'_>) -> Glyph {
    let glyph_metrics = icon_atlas.get_glyph_metrics(self.code_point).unwrap();

    Glyph::new(
      self.position,
      (glyph_metrics.size.0 as _, glyph_metrics.size.1 as _),
      self.color,
    )
    .tex_position((
      glyph_metrics.position.0 as _,
      glyph_metrics.position.1 as _,
      1.0,
    ))
    .call()
  }
}
