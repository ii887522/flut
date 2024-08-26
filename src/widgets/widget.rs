use super::{PainterWidget, Stack, StackChild, StatefulWidget, StatelessWidget};

#[derive(Debug)]
pub enum Widget<'a> {
  Stateless(Box<dyn StatelessWidget<'a> + 'a>),
  Stateful(Box<dyn StatefulWidget<'a> + 'a>),
  Painter(Box<dyn PainterWidget + 'a>),
  Stack(Stack<'a>),
  StackChild(Box<StackChild<'a>>),
}

impl Widget<'_> {
  pub(super) fn get_size(&self) -> (f32, f32) {
    match self {
      Widget::Stateless(widget) => widget.get_size(),
      Widget::Stateful(widget) => widget.get_size(),
      Widget::Painter(widget) => widget.get_size(),
      Widget::Stack(_) => (0.0, 0.0),
      Widget::StackChild(stack_child) => stack_child.get_size(),
    }
  }
}

pub trait FromStatelessWidget<T> {
  fn from_widget(widget: T) -> Self;
}

pub trait FromStatefulWidget<T> {
  fn from_widget(widget: T) -> Self;
}

pub trait FromPainterWidget<T> {
  fn from_widget(widget: T) -> Self;
}

pub trait IntoStatelessWidget<T> {
  fn into_widget(self) -> T;
}

pub trait IntoStatefulWidget<T> {
  fn into_widget(self) -> T;
}

pub trait IntoPainterWidget<T> {
  fn into_widget(self) -> T;
}

impl<'a, T: StatelessWidget<'a> + 'a> FromStatelessWidget<T> for Widget<'a> {
  fn from_widget(widget: T) -> Self {
    Self::Stateless(Box::new(widget))
  }
}

impl<'a, T: StatefulWidget<'a> + 'a> FromStatefulWidget<T> for Widget<'a> {
  fn from_widget(widget: T) -> Self {
    Self::Stateful(Box::new(widget))
  }
}

impl<'a, T: PainterWidget + 'a> FromPainterWidget<T> for Widget<'a> {
  fn from_widget(widget: T) -> Self {
    Self::Painter(Box::new(widget))
  }
}

impl<'a> From<Stack<'a>> for Widget<'a> {
  fn from(stack: Stack<'a>) -> Self {
    Self::Stack(stack)
  }
}

impl<'a, T: StatelessWidget<'a> + 'a> IntoStatelessWidget<Widget<'a>> for T {
  fn into_widget(self) -> Widget<'a> {
    Widget::Stateless(Box::new(self))
  }
}

impl<'a, T: StatefulWidget<'a> + 'a> IntoStatefulWidget<Widget<'a>> for T {
  fn into_widget(self) -> Widget<'a> {
    Widget::Stateful(Box::new(self))
  }
}

impl<'a, T: PainterWidget + 'a> IntoPainterWidget<Widget<'a>> for T {
  fn into_widget(self) -> Widget<'a> {
    Widget::Painter(Box::new(self))
  }
}
