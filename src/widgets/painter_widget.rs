use skia_safe::{Canvas, Rect};

pub trait PainterWidget {
  fn draw(&self, canvas: &Canvas, constraint: Rect);
}
