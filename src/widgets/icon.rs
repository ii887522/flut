use super::PainterWidget;
use crate::boot::context;
use optarg2chain::optarg_impl;
use skia_safe::{font::Edging, Canvas, Color, Font, Paint, Point, Rect};

#[derive(Debug, PartialEq)]
pub struct Icon {
  name: String,
  font: Font,
  color: Color,
  bound: Rect,
  degrees: f32,
}

#[optarg_impl]
impl Icon {
  #[optarg_method(IconNewBuilder, call)]
  pub fn new(
    name: u16,
    #[optarg(12.0)] size: f32,
    #[optarg(Color::BLACK)] color: Color,
    #[optarg_default] degrees: f32,
  ) -> Self {
    let name = String::from_utf16_lossy(&[name]);
    let mut font = context::ICON_TYPEFACE.with(|icon_typeface| Font::new(icon_typeface, size));
    font.set_edging(Edging::AntiAlias);
    let (_, bound) = font.measure_str(&name, None);

    Self {
      name,
      font,
      color,
      bound,
      degrees,
    }
  }
}

impl PainterWidget for Icon {
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

    if self.degrees != 0.0 {
      canvas.save();

      canvas.rotate(
        self.degrees,
        Some(Point::new(
          constraint.x() + self.bound.width() * 0.5,
          constraint.y() + self.bound.height() * 0.5,
        )),
      );
    }

    canvas.draw_str(
      &self.name,
      Point::new(
        constraint.x() - self.bound.x(),
        constraint.y() - self.bound.y(),
      ),
      &self.font,
      Paint::default().set_anti_alias(true).set_color(self.color),
    );

    if self.degrees != 0.0 {
      canvas.restore();
    }
  }

  fn get_size(&self) -> (f32, f32) {
    (self.bound.width(), self.bound.height())
  }
}
