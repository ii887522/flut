use super::PainterWidget;
use crate::boot::context;
use optarg2chain::optarg_impl;
use skia_safe::{font::Edging, Canvas, Color, Font, Paint, Point, Rect};

#[derive(Debug)]
pub struct Icon {
  name: String,
  font: Font,
  color: Color,
  bound: Rect,
}

#[optarg_impl]
impl Icon {
  #[optarg_method(IconNewBuilder, call)]
  pub fn new(name: u16, #[optarg(12.0)] size: f32, #[optarg(Color::BLACK)] color: Color) -> Self {
    let name = String::from_utf16_lossy(&[name]);
    let mut font = Font::new(&*context::ICON_TYPEFACE, size);
    font.set_edging(Edging::AntiAlias);
    let (_, bound) = font.measure_str(&name, None);

    Self {
      name,
      font,
      color,
      bound,
    }
  }
}

impl PainterWidget for Icon {
  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    canvas.draw_str(
      &self.name,
      Point::new(
        constraint.x() - self.bound.x(),
        constraint.y() - self.bound.y(),
      ),
      &self.font,
      Paint::default().set_anti_alias(true).set_color(self.color),
    );
  }

  fn get_size(&self) -> (f32, f32) {
    (self.bound.width(), self.bound.height())
  }
}
