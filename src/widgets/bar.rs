use super::{
  button::ButtonIcon, widget::*, Button, RectWidget, Stack, StackChild, StatelessWidget, Text,
  Widget,
};
use crate::{
  helpers::consts,
  models::{Origin, TextStyle},
};
use atomic_refcell::AtomicRefCell;
use skia_safe::{Color, FontStyle, Rect};
use std::{
  borrow::Cow,
  fmt::{self, Debug, Formatter},
  sync::Arc,
};

pub struct BarButton<'a> {
  pub is_enabled: bool,
  pub icon: u16,
  pub icon_color: Color,
  pub on_mouse_up: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
}

impl Debug for BarButton<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("BarButton")
      .field("is_enabled", &self.is_enabled)
      .field("icon", &self.icon)
      .field("icon_color", &self.icon_color)
      .finish_non_exhaustive()
  }
}

impl Default for BarButton<'_> {
  fn default() -> Self {
    Self {
      is_enabled: true,
      icon: 0,
      icon_color: Color::BLACK,
      on_mouse_up: Arc::new(AtomicRefCell::new(|| {})),
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct TitleStyle {
  pub font_family: &'static str,
  pub font_style: FontStyle,
  pub font_size: f32,
  pub color: Color,
}

impl Default for TitleStyle {
  fn default() -> Self {
    Self {
      font_family: consts::DEFAULT_FONT_FAMILY,
      font_style: FontStyle::default(),
      font_size: 32.0,
      color: Color::BLACK,
    }
  }
}

impl From<TitleStyle> for TextStyle {
  fn from(style: TitleStyle) -> Self {
    Self {
      font_family: style.font_family,
      font_style: style.font_style,
      font_size: style.font_size,
      color: style.color,
    }
  }
}

pub struct Bar<'a> {
  pub height: f32,
  pub color: Color,
  pub is_elevated: bool,
  pub leading_btn: BarButton<'a>,
  pub title: Cow<'static, str>,
  pub title_style: TitleStyle,
}

impl Default for Bar<'_> {
  fn default() -> Self {
    Self {
      height: 48.0,
      color: Color::BLACK,
      is_elevated: true,
      leading_btn: BarButton::default(),
      title: Cow::Borrowed(""),
      title_style: TitleStyle::default(),
    }
  }
}

impl<'a> StatelessWidget<'a> for Bar<'a> {
  fn get_size(&self) -> (f32, f32) {
    (-1.0, self.height)
  }

  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    Stack {
      children: vec![
        StackChild {
          position: (constraint.x(), constraint.y()),
          size: (constraint.width(), constraint.height()),
          origin: Origin::TopLeft,
          child: Some(
            RectWidget {
              color: self.color,
              border_radius: 0.0,
              is_elevated: self.is_elevated,
            }
            .into_widget(),
          ),
        },
        StackChild {
          position: (
            constraint.x() + 8.0,
            constraint.y() + constraint.height() * 0.5,
          ),
          size: (constraint.height(), constraint.height()),
          origin: Origin::Left,
          child: Some(
            Button {
              is_enabled: self.leading_btn.is_enabled,
              bg_color: Color::TRANSPARENT,
              border_radius: 100.0,
              is_elevated: false,
              icon: ButtonIcon::Icon {
                name: self.leading_btn.icon,
                color: self.leading_btn.icon_color,
              },
              on_mouse_up: Arc::clone(&self.leading_btn.on_mouse_up),
              ..Default::default()
            }
            .into_widget(),
          ),
        },
        StackChild {
          position: (
            constraint.x() + constraint.width() * 0.5,
            constraint.y() + constraint.height() * 0.5,
          ),
          size: (0.0, 0.0),
          origin: Origin::Center,
          child: Some(
            Text::new()
              .text(self.title.to_string())
              .style(self.title_style)
              .call()
              .into_widget(),
          ),
        },
      ],
    }
    .into()
  }
}
