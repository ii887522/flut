use super::{widget::*, Column, Spacing, StatelessWidget, Text, Widget};
use crate::{helpers, models::TextStyle};
use optarg2chain::optarg_impl;
use skia_safe::{Font, Rect};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct TextBlock {
  text: Cow<'static, str>,
  font: Font,
  style: TextStyle,
}

#[optarg_impl]
impl TextBlock {
  #[optarg_method(TextBlockNewBuilder, call)]
  pub fn new(
    #[optarg_default] text: Cow<'static, str>,
    #[optarg_default] style: TextStyle,
  ) -> Self {
    let font = helpers::new_font(style);
    Self { text, font, style }
  }
}

impl<'a> StatelessWidget<'a> for TextBlock {
  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    let mut text = String::with_capacity(self.text.len());
    let mut children = vec![];

    for (i, word) in self.text.split_whitespace().enumerate() {
      if i > 0 {
        text.push(' ');
      }

      text.push_str(word);
      let (_, bound) = self.font.measure_str(&text, None);

      if bound.width() <= constraint.width() {
        continue;
      }

      let Some((text_line, leftover)) = text.rsplit_once(' ') else {
        continue;
      };

      children.extend([
        Text::new()
          .text(text_line.to_string())
          .style(self.style)
          .call()
          .into_widget(),
        Spacing {
          height: self.font.size() * 0.5,
          ..Default::default()
        }
        .into_widget(),
      ]);

      let leftover = leftover.to_string();
      text.clear();
      text.push_str(&leftover);
    }

    children.push(
      Text::new()
        .text(text)
        .style(self.style)
        .call()
        .into_widget(),
    );

    Column::new().children(children).call().into_widget()
  }
}
