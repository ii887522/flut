use super::Widget;
use skia_safe::Rect;

#[derive(Clone)]
pub struct StackChild<'a> {
  pub position: (f32, f32),
  pub size: (f32, f32),
  pub child: Widget<'a>,
}

impl<'a> StackChild<'a> {
  pub fn new(constraint: Rect, child: Widget<'a>) -> Self {
    Self {
      position: (constraint.x(), constraint.y()),
      size: (constraint.width(), constraint.height()),
      child,
    }
  }
}
