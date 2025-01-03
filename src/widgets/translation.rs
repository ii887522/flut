use super::{BuilderWidget, Widget};
use skia_safe::{Canvas, Rect};

pub struct Translation<'a> {
  pub translation: (f32, f32),
  pub child: Widget<'a>,
}

impl<'a> BuilderWidget<'a> for Translation<'a> {
  fn pre_draw(&self, canvas: &Canvas, _constraint: Rect) {
    canvas.save();
    canvas.translate(self.translation);
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    Widget::clone(&self.child)
  }

  fn post_draw(&self, canvas: &Canvas, _constraint: Rect) {
    canvas.restore();
  }
}
