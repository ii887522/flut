use skia_safe::{Canvas, Rect};
use std::fmt::Debug;

pub trait PainterWidget: Debug + Send {
  fn draw(&self, canvas: &Canvas, constraint: Rect);

  fn get_size(&self) -> (f32, f32) {
    (0.0, 0.0)
  }
}
