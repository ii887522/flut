use super::{BuilderWidget, PainterWidget, Stack};

pub enum Widget<'a> {
  Builder(Box<dyn BuilderWidget<'a> + 'a + Send + Sync>),
  Painter(Box<dyn PainterWidget + 'a + Send + Sync>),
  Stack(Stack<'a>),
}

impl Widget<'_> {
  pub fn get_size(&self) -> (f32, f32) {
    match self {
      Widget::Builder(widget) => widget.get_size(),
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
    Widget::Builder(Box::new(widget))
  }
}

impl<'a> FromPainterWidget<'a> for Widget<'a> {
  fn from_widget(widget: impl PainterWidget + 'a + Send + Sync) -> Self {
    Widget::Painter(Box::new(widget))
  }
}

impl<'a> From<Stack<'a>> for Widget<'a> {
  fn from(stack: Stack<'a>) -> Self {
    Widget::Stack(stack)
  }
}

impl<'a, W: BuilderWidget<'a> + 'a + Send + Sync> IntoBuilderWidget<'a> for W {
  fn into_widget(self) -> Widget<'a> {
    Widget::Builder(Box::new(self))
  }
}

impl<'a, W: PainterWidget + 'a + Send + Sync> IntoPainterWidget<'a> for W {
  fn into_widget(self) -> Widget<'a> {
    Widget::Painter(Box::new(self))
  }
}
