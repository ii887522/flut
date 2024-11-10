use super::PainterWidget;
use crate::{boot::context, models::FontCfg};
use optarg2chain::optarg_impl;
use skia_safe::{Canvas, Color, Font, FontStyle, Paint, Point, Rect};
use std::borrow::Cow;

pub struct Text {
  text: Cow<'static, str>,
  font_cfg: FontCfg,
  bound: Rect,
  color: Color,
}

#[optarg_impl]
impl Text {
  #[optarg_method(TextNewBuilder, call)]
  pub fn new<'a>(
    #[optarg_default] text: Cow<'static, str>,
    #[optarg_default] font_cfg: FontCfg,
    #[optarg(Color::BLACK)] color: Color,
  ) -> Self {
    context::FONT_CACHE.with_borrow_mut(|font_cache| {
      let font = font_cache.entry(font_cfg).or_insert_with(|| {
        let font_style = FontStyle::new(
          font_cfg.font_weight,
          font_cfg.font_width,
          font_cfg.font_slant,
        );

        let typeface = context::FONT_MGR.with(|font_mgr| {
          font_mgr
            .match_family_style(font_cfg.font_family, font_style)
            .unwrap()
        });

        Font::new(typeface, font_cfg.font_size as f32)
      });

      let (_, bound) = font.measure_str(&text, None);

      Self {
        font_cfg,
        text,
        bound,
        color,
      }
    })
  }
}

impl PainterWidget for Text {
  fn get_size(&self) -> (f32, f32) {
    (self.bound.width(), self.bound.height())
  }

  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    // Uncomment to draw boundary for debugging purpose
    // canvas.draw_rect(
    //   Rect::from_xywh(
    //     constraint.x(),
    //     constraint.y(),
    //     self.bound.width(),
    //     self.bound.height(),
    //   ),
    //   Paint::default()
    //     .set_anti_alias(true)
    //     .set_stroke(true)
    //     .set_stroke_width(2.0)
    //     .set_color(Color::MAGENTA),
    // );

    context::FONT_CACHE.with_borrow(|font_cache| {
      canvas.draw_str(
        &self.text,
        Point::new(
          constraint.x() - self.bound.x(),
          constraint.y() - self.bound.y(),
        ),
        &font_cache[&self.font_cfg],
        Paint::default().set_anti_alias(true).set_color(self.color),
      );
    });
  }
}
