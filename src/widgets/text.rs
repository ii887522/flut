use super::PainterWidget;
use crate::{
  helpers::{self},
  models::{Lang, Origin, TextStyle},
};
use optarg2chain::optarg_impl;
use skia_safe::{font::Edging, Canvas, Font, Paint, Point, Rect};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct Text {
  text: Cow<'static, str>,
  font: Font,
  style: TextStyle,
  bound: Rect,
}

#[optarg_impl]
impl Text {
  #[optarg_method(TextNewBuilder, call)]
  pub fn new(
    #[optarg_default] text: Cow<'static, str>,
    #[optarg_default] style: TextStyle,
  ) -> Self {
    let mut font = helpers::new_font(style);
    font.set_edging(Edging::AntiAlias);
    let (_, bound) = font.measure_str(&text, None);

    Self {
      text,
      font,
      style,
      bound,
    }
  }
}

impl PainterWidget for Text {
  fn get_size(&self) -> (f32, f32) {
    (self.bound.width(), self.font.size() * 0.75)
  }

  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    // Uncomment to draw boundary for debugging purpose
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

    let y_offset = match Lang::from_font_family(self.style.font_family).get_text_origin() {
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
      Paint::default()
        .set_anti_alias(true)
        .set_color(self.style.color),
    );
  }
}
