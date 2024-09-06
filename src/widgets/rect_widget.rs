use super::PainterWidget;
use skia_safe::{Canvas, Color, Paint, RRect, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectWidget {
  pub color: Color,
  pub border_radius: f32,
}

impl Default for RectWidget {
  fn default() -> Self {
    Self {
      color: Color::BLACK,
      border_radius: 0.0,
    }
  }
}

impl PainterWidget for RectWidget {
  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    canvas.draw_rrect(
      RRect::new_rect_xy(constraint, self.border_radius, self.border_radius),
      Paint::default().set_anti_alias(true).set_color(self.color),
    );
  }
}
