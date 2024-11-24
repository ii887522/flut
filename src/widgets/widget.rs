use super::{BuilderWidget, PainterWidget, Stack};
use atomic_refcell::AtomicRefCell;
use std::sync::Arc;

#[derive(Clone)]
pub enum Widget<'a> {
  Builder(Arc<AtomicRefCell<dyn BuilderWidget<'a> + 'a + Send + Sync>>),
  Painter(Arc<dyn PainterWidget + 'a + Send + Sync>),
  Stack(Arc<Stack<'a>>),
}

impl Widget<'_> {
  pub fn get_size(&self) -> (f32, f32) {
    match self {
      Widget::Builder(widget) => widget.borrow().get_size(),
      Widget::Painter(widget) => widget.get_size(),
      Widget::Stack(_) => (-1.0, -1.0),
    }
  }
}

pub trait FromBuilderWidget<'a> {
  fn from_widget(widget: impl BuilderWidget<'a> + 'a + Send + Sync) -> Self;
}

pub trait FromPainterWidget<'a> {
  fn from_widget(widget: impl PainterWidget + 'a + Send + Sync) -> Self;
}

pub trait IntoBuilderWidget<'a> {
  fn into_widget(self) -> Widget<'a>;
}

pub trait IntoPainterWidget<'a> {
  fn into_widget(self) -> Widget<'a>;
}

impl<'a> FromBuilderWidget<'a> for Widget<'a> {
  fn from_widget(widget: impl BuilderWidget<'a> + 'a + Send + Sync) -> Self {
    Self::Builder(Arc::new(AtomicRefCell::new(widget)))
  }
}

impl<'a> FromPainterWidget<'a> for Widget<'a> {
  fn from_widget(widget: impl PainterWidget + 'a + Send + Sync) -> Self {
    Self::Painter(Arc::new(widget))
  }
}

impl<'a> From<Stack<'a>> for Widget<'a> {
  fn from(stack: Stack<'a>) -> Self {
    Self::Stack(Arc::new(stack))
  }
}

impl<'a, W: BuilderWidget<'a> + 'a + Send + Sync> IntoBuilderWidget<'a> for W {
  fn into_widget(self) -> Widget<'a> {
    Widget::Builder(Arc::new(AtomicRefCell::new(self)))
  }
}

impl<'a, W: PainterWidget + 'a + Send + Sync> IntoPainterWidget<'a> for W {
  fn into_widget(self) -> Widget<'a> {
    Widget::Painter(Arc::new(self))
  }
}
