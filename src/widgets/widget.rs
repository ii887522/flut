use super::{PainterWidget, Stack, StackChild, StatefulWidget, StatelessWidget};

#[derive(Debug)]
pub enum Widget<'a> {
  Stateless(Box<dyn StatelessWidget + 'a>),
  Stateful(Box<dyn StatefulWidget + 'a>),
  Painter(Box<dyn PainterWidget + 'a>),
  Stack(Stack<'a>),
  StackChild(Box<StackChild<'a>>),
}
