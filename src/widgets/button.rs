use super::{
  widget::*, Icon, RectWidget, Row, Spacing, Stack, StackChild, StatelessWidget, Text, Widget,
};
use crate::models::{Origin, VerticalAlign};
use skia_safe::{
  font_style::{Slant, Weight, Width},
  Color, FontStyle, Rect,
};

#[derive(Debug, PartialEq)]
pub struct Button {
  pub bg_color: Color,
  pub border_radius: f32,
  pub is_elevated: bool,
  pub icon: u16,
  pub icon_color: Color,
  pub label: String,
  pub label_color: Color,
}

impl Default for Button {
  fn default() -> Self {
    Self {
      bg_color: Color::WHITE,
      border_radius: 8.0,
      is_elevated: true,
      icon: 0,
      icon_color: Color::BLACK,
      label: "".to_string(),
      label_color: Color::BLACK,
    }
  }
}

impl<'a> StatelessWidget<'a> for Button {
  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    Stack {
      children: vec![
        StackChild {
          position: (constraint.x(), constraint.y()),
          size: (constraint.width(), constraint.height()),
          origin: Origin::TopLeft,
          child: Some(
            RectWidget {
              color: self.bg_color,
              border_radius: self.border_radius,
              is_elevated: self.is_elevated,
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
            Row::new()
              // Somehow VerticalAlign::Middle looks like align slightly towards the top,
              // have to use VerticalAlign::Bottom as the workaround
              .align(VerticalAlign::Bottom)
              .children(
                vec![
                  if self.icon == 0 {
                    None
                  } else {
                    Some(
                      Icon::new(self.icon)
                        .size(40.0)
                        .color(self.icon_color)
                        .call()
                        .into_widget(),
                    )
                  },
                  if self.icon == 0 || self.label.is_empty() {
                    None
                  } else {
                    Some(
                      Spacing {
                        width: 16.0,
                        ..Default::default()
                      }
                      .into_widget(),
                    )
                  },
                  if self.label.is_empty() {
                    None
                  } else {
                    Some(
                      Text::new()
                        .text(self.label.to_string())
                        .font_size(28.0)
                        .font_style(FontStyle::new(
                          Weight::SEMI_BOLD,
                          Width::NORMAL,
                          Slant::Upright,
                        ))
                        .color(self.label_color)
                        .call()
                        .into_widget(),
                    )
                  },
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
              )
              .call()
              .into_widget(),
          ),
        },
      ],
    }
    .into()
  }
}
