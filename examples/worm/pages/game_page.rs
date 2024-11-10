use flut::{
  models::{FontCfg, HorizontalAlign},
  widgets::{widget::*, BuilderWidget, Column, Grid, RectWidget, Spacing, Text, Widget},
};
use skia_safe::{Color, Rect};

pub(crate) struct GamePage;

impl<'a> BuilderWidget<'a> for GamePage {
  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    Column {
      align: HorizontalAlign::Center,
      children: vec![
        Spacing {
          height: 16.0,
          ..Default::default()
        }
        .into_widget(),
        Text::new()
          .text("0")
          .font_cfg(FontCfg {
            font_size: 48,
            ..Default::default()
          })
          .color(Color::WHITE)
          .call()
          .into_widget(),
        Spacing {
          height: 16.0,
          ..Default::default()
        }
        .into_widget(),
        Grid {
          col_count: 41,
          row_count: 41,
          gap: 2.0,
          builder: Box::new(|_| {
            RectWidget {
              color: Color::DARK_GRAY,
            }
            .into_widget()
          }),
        }
        .into_widget(),
      ],
    }
    .into_widget()
  }
}
