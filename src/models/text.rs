use super::{Anchor, Glyph};
use crate::font_atlas::FontAtlas;
use optarg2chain::optarg_impl;
use rayon::prelude::*;
use std::borrow::Cow;

#[derive(Clone)]
pub struct Text {
  pub(super) position: (f32, f32),
  pub(super) color: (u8, u8, u8),
  pub(super) text: Cow<'static, str>,
  pub(super) anchor: Anchor,
}

#[optarg_impl]
impl Text {
  #[optarg_method(TextNewBuilder, call)]
  pub fn new(
    position: (f32, f32),
    color: (u8, u8, u8),
    text: Cow<'static, str>,
    #[optarg_default] anchor: Anchor,
  ) -> Self {
    Self {
      position,
      color,
      text,
      anchor,
    }
  }

  pub(crate) fn into_glyphs(self, font_atlas: &FontAtlas) -> Vec<Glyph> {
    let mut last_glyph_position = self.position;
    let mut current_glyph_position = self.position;
    let mut last_glyph_width = 0;
    let mut max_glyph_height = 0;

    let glyphs = self
      .text
      .chars()
      .map(|char| {
        let glyph_metrics = font_atlas.get_glyph_metrics(char).unwrap();

        let glyph = Glyph::new(
          current_glyph_position,
          (glyph_metrics.size.0 as _, glyph_metrics.size.1 as _),
          self.color,
        )
        .tex_position((glyph_metrics.position.0 as _, glyph_metrics.position.1 as _))
        .call();

        last_glyph_position = current_glyph_position;
        current_glyph_position.0 += glyph_metrics.advance as f32;
        last_glyph_width = glyph_metrics.size.0;
        max_glyph_height = max_glyph_height.max(glyph_metrics.size.1);

        glyph
      })
      .collect::<Vec<_>>();

    let text_size = (
      last_glyph_position.0 + last_glyph_width as f32 - self.position.0,
      max_glyph_height as f32,
    );

    let offset = match self.anchor {
      Anchor::TopLeft => (0.0, 0.0),
      Anchor::Top => (-text_size.0 * 0.5, 0.0),
      Anchor::TopRight => (-text_size.0, 0.0),
      Anchor::Left => (0.0, -text_size.1 * 0.5),
      Anchor::Center => (-text_size.0 * 0.5, -text_size.1 * 0.5),
      Anchor::Right => (-text_size.0, -text_size.1 * 0.5),
      Anchor::BottomLeft => (0.0, -text_size.1),
      Anchor::Bottom => (-text_size.0 * 0.5, -text_size.1),
      Anchor::BottomRight => (-text_size.0, -text_size.1),
    };

    glyphs
      .into_par_iter()
      .map(|glyph| {
        Glyph::new(
          (glyph.position.0 + offset.0, glyph.position.1 + offset.1),
          glyph.size,
          crate::unpack_color(glyph.color),
        )
        .tex_position(glyph.tex_position)
        .call()
      })
      .collect()
  }
}
