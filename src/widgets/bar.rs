use super::{widget::*, Button, RectWidget, Stack, StackChild, StatelessWidget, Text, Widget};
use crate::models::{icon_name, Origin, TextStyle};
use skia_safe::{Color, Rect};
use std::{
  borrow::Cow,
  sync::{Arc, Mutex},
};

#[derive(Debug, PartialEq)]
pub struct Bar {
  pub height: f32,
  pub color: Color,
  pub is_elevated: bool,
  pub title: Cow<'static, str>,
  pub title_style: TextStyle,
}

impl Default for Bar {
  fn default() -> Self {
    Self {
      height: 48.0,
      color: Color::BLACK,
      is_elevated: true,
      title: Cow::Borrowed(""),
      title_style: TextStyle {
        font_size: 32.0,
        ..Default::default()
      },
    }
  }
}

impl<'a> StatelessWidget<'a> for Bar {
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
              bg_color: Color::TRANSPARENT,
              border_radius: 100.0,
              is_elevated: false,
              icon: icon_name::ARROW_BACK,
              icon_color: Color::from_rgb(128, 0, 0),
              on_mouse_up: Arc::new(Mutex::new(|| {})),
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
