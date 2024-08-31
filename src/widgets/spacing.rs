use super::PainterWidget;
use skia_safe::{Canvas, Rect};

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Spacing {
  pub width: f32,
  pub height: f32,
}

impl PainterWidget for Spacing {
  fn draw(&self, _canvas: &Canvas, _constraint: Rect) {
    // This widget is served for the purpose of separating between widgets for better visual
  }

  fn get_size(&self) -> (f32, f32) {
    (self.width, self.height)
  }
}
