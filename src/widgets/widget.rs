use super::{PainterWidget, Stack, StackChild, StatefulWidget, StatelessWidget};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum Widget<'a> {
  Stateless(Arc<Mutex<dyn StatelessWidget<'a> + 'a + Sync>>),
  Stateful(Arc<Mutex<dyn StatefulWidget<'a> + 'a + Sync>>),
  Painter(Arc<dyn PainterWidget + 'a + Sync>),
  Stack(Arc<Mutex<Stack<'a>>>),
  StackChild(Box<StackChild<'a>>),
}

impl Widget<'_> {
  pub(super) fn get_size(&self) -> (f32, f32) {
    match self {
      Widget::Stateless(widget) => widget.lock().unwrap().get_size(),
      Widget::Stateful(widget) => widget.lock().unwrap().get_size(),
      Widget::Painter(widget) => widget.get_size(),
      Widget::Stack(_) => (-1.0, -1.0),
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

impl<'a, T: StatelessWidget<'a> + 'a + Sync> FromStatelessWidget<T> for Widget<'a> {
  fn from_widget(widget: T) -> Self {
    Self::Stateless(Arc::new(Mutex::new(widget)))
  }
}

impl<'a, T: StatefulWidget<'a> + 'a + Sync> FromStatefulWidget<T> for Widget<'a> {
  fn from_widget(widget: T) -> Self {
    Self::Stateful(Arc::new(Mutex::new(widget)))
  }
}

impl<'a, T: PainterWidget + 'a + Sync> FromPainterWidget<T> for Widget<'a> {
  fn from_widget(widget: T) -> Self {
    Self::Painter(Arc::new(widget))
  }
}

impl<'a> From<Stack<'a>> for Widget<'a> {
  fn from(stack: Stack<'a>) -> Self {
    Self::Stack(Arc::new(Mutex::new(stack)))
  }
}

impl<'a, T: StatelessWidget<'a> + 'a + Sync> IntoStatelessWidget<Widget<'a>> for T {
  fn into_widget(self) -> Widget<'a> {
    Widget::Stateless(Arc::new(Mutex::new(self)))
  }
}

impl<'a, T: StatefulWidget<'a> + 'a + Sync> IntoStatefulWidget<Widget<'a>> for T {
  fn into_widget(self) -> Widget<'a> {
    Widget::Stateful(Arc::new(Mutex::new(self)))
  }
}

impl<'a, T: PainterWidget + 'a + Sync> IntoPainterWidget<Widget<'a>> for T {
  fn into_widget(self) -> Widget<'a> {
    Widget::Painter(Arc::new(self))
  }
}
