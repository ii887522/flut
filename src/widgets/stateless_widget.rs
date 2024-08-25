use super::Widget;
use skia_safe::{Canvas, Rect};
use std::fmt::Debug;

pub trait StatelessWidget: Debug + Send {
  fn pre_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
  fn build<'a>(&self, constraint: Rect) -> Widget<'a>;
  fn post_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
}
