use super::PainterWidget;
use crate::{
  boot::context,
  models::{Lang, Origin},
};
use optarg2chain::optarg_impl;
use skia_safe::{font::Edging, Canvas, Color, Font, FontStyle, Paint, Point, Rect};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct Text {
  text: Cow<'static, str>,
  font: Font,
  font_family: &'static str,
  color: Color,
  bound: Rect,
}

#[optarg_impl]
impl Text {
  #[optarg_method(TextNewBuilder, call)]
  pub fn new(
    #[optarg_default] text: Cow<'static, str>,
    #[optarg("Arial")] font_family: &'static str,
    #[optarg_default] font_style: FontStyle,
    #[optarg(12.0)] font_size: f32,
    #[optarg(Color::BLACK)] color: Color,
  ) -> Self {
    let mut font = context::TEXT_TYPEFACES.with_borrow_mut(|text_typefaces| {
      let typeface = text_typefaces
        .entry(format!(
          "FontFamily#{}#Weight#{}#Width#{}#Slant#{:?}",
          font_family,
          *font_style.weight(),
          *font_style.width(),
          font_style.slant()
        ))
        .or_insert_with(|| {
          context::FONT_MGR.with(|font_mgr| {
            font_mgr
              .match_family_style(font_family, font_style)
              .unwrap()
          })
        });

      Font::new(&*typeface, font_size)
    });

    font.set_edging(Edging::AntiAlias);
    let (_, bound) = font.measure_str(&text, None);

    Self {
      text,
      font,
      font_family,
      color,
      bound,
    }
  }
}

impl PainterWidget for Text {
  fn get_size(&self) -> (f32, f32) {
    (self.bound.width(), self.font.size() * 0.75)
  }

  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    // Draw boundary for debugging purpose
    // canvas.draw_rect(
    //   Rect::from_xywh(
    //     constraint.x(),
    //     constraint.y(),
    //     self.bound.width(),
    //     self.font.size() * 0.75,
    //   ),
    //   Paint::default()
    //     .set_anti_alias(true)
    //     .set_stroke(true)
    //     .set_stroke_width(2.0)
    //     .set_color(Color::MAGENTA),
    // );

    let y_offset = match Lang::from_font_family(self.font_family).get_text_origin() {
      Origin::TopLeft => 0.0,
      Origin::Left => (self.bound.height() - self.font.size() * 0.75) * 0.5,
      origin => unreachable!("{origin:?} not yet supported in Text::draw() implementation"),
    };

    canvas.draw_str(
      &self.text,
      Point::new(
        constraint.x() - self.bound.x(),
        constraint.y() - self.bound.y() - y_offset,
      ),
      &self.font,
      Paint::default().set_anti_alias(true).set_color(self.color),
    );
  }
}
