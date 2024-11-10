use super::PainterWidget;
use skia_safe::{Canvas, Color, Paint, Rect};

pub struct RectWidget {
  pub color: Color,
}

impl Default for RectWidget {
  fn default() -> Self {
    Self {
      color: Color::BLACK,
    }
  }
}

impl PainterWidget for RectWidget {
  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    canvas.draw_rect(
      constraint,
      Paint::default().set_anti_alias(true).set_color(self.color),
    );
  }
}
