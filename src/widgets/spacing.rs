use super::PainterWidget;
use skia_safe::{Canvas, Rect};

pub struct Spacing {
  pub width: f32,
  pub height: f32,
}

impl Default for Spacing {
  fn default() -> Self {
    Self {
      width: -1.0,
      height: -1.0,
    }
  }
}

impl PainterWidget for Spacing {
  fn get_size(&self) -> (f32, f32) {
    (self.width, self.height)
  }

  fn draw(&self, _canvas: &Canvas, _constraint: Rect) {
    // No drawing needed because this widget is used to separate adjacent widgets for better visual
  }
}
