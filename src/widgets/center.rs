use super::{Stack, StackChild, StatelessWidget, Widget};
use crate::models::Origin;
use skia_safe::Rect;

#[derive(Default)]
pub struct Center<'a> {
  pub child: Option<Widget<'a>>,
}

impl<'a> StatelessWidget<'a> for Center<'a> {
  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    Stack {
      children: vec![StackChild {
        position: (
          constraint.x() + constraint.width() * 0.5,
          constraint.y() + constraint.height() * 0.5,
        ),
        size: (constraint.width(), constraint.height()),
        origin: Origin::Center,
        child: self.child.take(),
      }],
    }
    .into()
  }
}
