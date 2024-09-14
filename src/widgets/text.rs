use super::PainterWidget;
use crate::boot::context;
use optarg2chain::optarg_impl;
use skia_safe::{font::Edging, Canvas, Color, Font, FontStyle, Paint, Point, Rect};

#[derive(Debug, PartialEq)]
pub struct Text {
  text: String,
  font: Font,
  color: Color,
  bound: Rect,
}

#[optarg_impl]
impl Text {
  #[optarg_method(TextNewBuilder, call)]
  pub fn new<'a>(
    #[optarg_default] text: String,
    #[optarg("Segoe UI")] font_family: &'a str,
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
        .or_insert_with_key(|_| {
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
      color,
      bound,
    }
  }
}

impl PainterWidget for Text {
  fn get_size(&self) -> (f32, f32) {
    (self.bound.width(), self.bound.height())
  }

  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    canvas.draw_str(
      &self.text,
      Point::new(
        constraint.x() - self.bound.x(),
        constraint.y() - self.bound.y(),
      ),
      &self.font,
      Paint::default().set_anti_alias(true).set_color(self.color),
    );
  }
}
