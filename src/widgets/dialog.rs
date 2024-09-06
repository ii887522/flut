use super::{widget::*, RectWidget, Stack, StackChild, StatelessWidget, Widget};
use crate::{
  boot::context,
  widgets::{Icon, Text},
};
use skia_safe::{Color, Rect};
use std::sync::atomic::Ordering;

#[derive(Debug, PartialEq, Eq)]
pub struct Dialog {
  pub color: Color,
  pub header_icon: u16,
  pub header_icon_color: Color,
  pub header_title: String,
  pub header_title_color: Color,
}

impl Default for Dialog {
  fn default() -> Self {
    Self {
      color: Color::BLACK,
      header_icon: 0,
      header_icon_color: Color::BLACK,
      header_title: "".to_string(),
      header_title_color: Color::BLACK,
    }
  }
}

impl<'a> StatelessWidget<'a> for Dialog {
  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    const SIZE: (f32, f32) = (600.0, 300.0);

    let drawable_size = (
      context::DRAWABLE_SIZE.0.load(Ordering::Relaxed),
      context::DRAWABLE_SIZE.1.load(Ordering::Relaxed),
    );

    let position = (
      (drawable_size.0 - SIZE.0) * 0.5,
      (drawable_size.1 - SIZE.1) * 0.5,
    );

    Stack {
      children: vec![
        Some(StackChild {
          position: (0.0, 0.0),
          size: drawable_size,
          child: Some(
            RectWidget {
              color: Color::from_argb(128, 0, 0, 0),
              ..Default::default()
            }
            .into_widget(),
          ),
        }),
        Some(StackChild {
          position,
          size: SIZE,
          child: Some(
            RectWidget {
              color: self.color,
              border_radius: 8.0,
            }
            .into_widget(),
          ),
        }),
        if self.header_icon == 0 {
          None
        } else {
          Some(StackChild {
            position,
            size: (0.0, 0.0),
            child: Some(
              Icon::new(self.header_icon)
                .size(64.0)
                .color(self.header_icon_color)
                .call()
                .into_widget(),
            ),
          })
        },
        if self.header_title.is_empty() {
          None
        } else {
          Some(StackChild {
            position: (position.0 + 64.0, position.1 + 16.0),
            size: (0.0, 0.0),
            child: Some(
              Text::new()
                .text(&self.header_title)
                .font_size(32.0)
                .color(self.header_title_color)
                .call()
                .into_widget(),
            ),
          })
        },
      ]
      .into_iter()
      .flatten()
      .collect(),
    }
    .into()
  }

  fn get_size(&self) -> (f32, f32) {
    // (0.0, 0.0) so that this widget can be inserted in Column or Row or any other layout widget.
    // Size is ignored and this widget always cover the whole app
    (0.0, 0.0)
  }
}
