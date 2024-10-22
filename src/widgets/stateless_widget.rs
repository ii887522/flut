use super::Widget;
use skia_safe::{Canvas, Rect};

pub trait StatelessWidget<'a>: Send {
  fn get_size(&self) -> (f32, f32) {
    (-1.0, -1.0)
  }

  fn pre_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
  fn build(&mut self, constraint: Rect) -> Widget<'a>;
  fn post_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
}
