use super::{Anchor, Glyph, Glyphs};
use crate::atlases::FontAtlas;
use optarg2chain::optarg_impl;
use rayon::prelude::*;
use std::borrow::Cow;

#[derive(Clone)]
pub struct Text {
  pub(super) position: (f32, f32, f32),
  pub(super) font_size: f32,
  pub(super) max_width: f32,
  pub(super) color: (u8, u8, u8, u8),
  pub(super) text: Cow<'static, str>,
  pub(super) anchor: Anchor,
}

#[optarg_impl]
impl Text {
  #[optarg_method(TextNewBuilder, call)]
  pub fn new(
    position: (f32, f32, f32),
    font_size: f32,
    #[optarg(f32::MAX)] max_width: f32,
    color: (u8, u8, u8, u8),
    text: Cow<'static, str>,
    #[optarg_default] anchor: Anchor,
  ) -> Self {
    Self {
      position,
      font_size,
      max_width,
      color,
      text,
      anchor,
    }
  }

  pub(crate) fn into_glyphs(self, font_atlas: &FontAtlas) -> Glyphs {
    let font_scale = self.font_size / font_atlas.font_size as f32;
    let mut last_glyph_position = self.position;
    let mut current_glyph_position = self.position;
    let mut last_glyph_width: f32 = 0.0;
    let mut max_glyph_height: f32 = 0.0;
    let mut text_size: (f32, f32) = (0.0, 0.0);

    let glyphs = self
      .text
      .split_whitespace()
      .flat_map(|word| {
        let glyphs = loop {
          let glyphs = word
            .chars()
            .map(|char| {
              let glyph_metrics = font_atlas.get_glyph_metrics(char).unwrap();

              let glyph = Glyph::new(
                current_glyph_position,
                (
                  glyph_metrics.size.0 as f32 * font_scale,
                  glyph_metrics.size.1 as f32 * font_scale,
                ),
                self.color,
              )
              .tex_position((
                glyph_metrics.position.0 as _,
                glyph_metrics.position.1 as _,
                0.0,
              ))
              .tex_size((glyph_metrics.size.0 as _, glyph_metrics.size.1 as _))
              .call();

              last_glyph_position = current_glyph_position;
              current_glyph_position.0 += glyph_metrics.advance as f32 * font_scale;
              last_glyph_width = glyph_metrics.size.0 as f32 * font_scale;
              max_glyph_height = max_glyph_height.max(glyph_metrics.size.1 as f32 * font_scale);

              glyph
            })
            .collect::<Vec<_>>();

          if last_glyph_position.0 + last_glyph_width <= self.position.0 + self.max_width {
            break glyphs;
          }

          current_glyph_position.0 = self.position.0;
          current_glyph_position.1 += font_atlas.recommended_line_spacing * font_scale;
          max_glyph_height = 0.0;
        };

        text_size.0 = text_size
          .0
          .max(last_glyph_position.0 + last_glyph_width - self.position.0);

        let glyph_metrics = font_atlas.get_glyph_metrics(' ').unwrap();
        current_glyph_position.0 += glyph_metrics.advance as f32 * font_scale;

        glyphs
      })
      .collect::<Vec<_>>();

    text_size.1 = last_glyph_position.1 + max_glyph_height - self.position.1;
    let offset = crate::calc_position_offset(self.anchor, text_size);

    let glyphs = glyphs
      .into_par_iter()
      .map(|glyph| {
        Glyph::new(
          (
            glyph.position.0 + offset.0,
            glyph.position.1 + offset.1,
            glyph.position.2,
          ),
          glyph.size,
          crate::unpack_color(glyph.color),
        )
        .tex_position(glyph.tex_position)
        .tex_size(glyph.tex_size)
        .call()
      })
      .collect();

    Glyphs { glyphs, text_size }
  }
}
