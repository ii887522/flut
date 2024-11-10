use skia_safe::{Canvas, Rect};

pub trait PainterWidget {
  fn get_size(&self) -> (f32, f32) {
    (-1.0, -1.0)
  }

  fn draw(&self, canvas: &Canvas, constraint: Rect);
}
