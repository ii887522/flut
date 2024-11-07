use super::{BuilderWidget, PainterWidget, Stack};

pub enum Widget<'a> {
  Builder(Box<dyn BuilderWidget<'a> + 'a + Send + Sync>),
  Painter(Box<dyn PainterWidget + 'a + Send + Sync>),
  Stack(Stack<'a>),
}
