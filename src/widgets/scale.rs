use super::{BuilderWidget, Widget};
use skia_safe::{Canvas, Rect};

pub struct Scale<'a> {
  pub scale: f32,
  pub child: Widget<'a>,
}

impl<'a> BuilderWidget<'a> for Scale<'a> {
  fn pre_draw(&self, canvas: &Canvas, constraint: Rect) {
    canvas.save();
    canvas.translate(constraint.center());
    canvas.scale((self.scale, self.scale));
    canvas.translate(-constraint.center());
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    Widget::clone(&self.child)
  }

  fn post_draw(&self, canvas: &Canvas, _constraint: Rect) {
    canvas.restore();
  }
}
