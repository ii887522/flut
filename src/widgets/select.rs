use super::{StatelessWidget, Widget};
use skia_safe::Rect;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Select;

impl<'a> StatelessWidget<'a> for Select {
  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    todo!()
  }
}
