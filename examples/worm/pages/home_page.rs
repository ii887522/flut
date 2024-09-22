use flut::{
  models::{icon_name, HorizontalAlign, VerticalAlign},
  widgets::{
    router::Navigator, widget::*, Button, Column, ImageWidget, Row, Spacing, StatelessWidget, Text,
    Widget,
  },
};
use skia_safe::{Color, Rect};
use std::{
  process,
  sync::{Arc, Mutex},
};

#[derive(Debug)]
pub(crate) struct HomePage {
  pub(crate) navigator: Arc<Mutex<Navigator>>,
}

impl<'a> StatelessWidget<'a> for HomePage {
  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let navigator = Arc::clone(&self.navigator);

    Column::new()
      .align(HorizontalAlign::Center)
      .children(vec![
        Spacing {
          height: 128.0,
          ..Default::default()
        }
        .into_widget(),
        Row::new()
          .align(VerticalAlign::Middle)
          .children(vec![
            ImageWidget::new("assets/images/favicon.png")
              .size((64.0, 64.0))
              .call()
              .into_widget(),
            Spacing {
              width: 24.0,
              ..Default::default()
            }
            .into_widget(),
            Text::new()
              .text("Worm".to_string())
              .color(Color::from_rgb(243, 125, 121))
              .font_size(64.0)
              .call()
              .into_widget(),
          ])
          .call()
          .into_widget(),
        Spacing {
          height: 256.0,
          ..Default::default()
        }
        .into_widget(),
        Button {
          bg_color: Color::GREEN,
          icon: icon_name::PLAY_ARROW,
          label: "Start Game".to_string(),
          size: (256.0, 64.0),
          on_mouse_up: Arc::new(Mutex::new(move || {
            let mut navigator = navigator.lock().unwrap();
            navigator.go("/game".to_string());
          })),
          ..Default::default()
        }
        .into_widget(),
        Spacing {
          height: 64.0,
          ..Default::default()
        }
        .into_widget(),
        Button {
          bg_color: Color::RED,
          icon: icon_name::LOGOUT,
          label: "Exit Game".to_string(),
          size: (256.0, 64.0),
          on_mouse_up: Arc::new(Mutex::new(|| process::exit(0))),
          ..Default::default()
        }
        .into_widget(),
      ])
      .call()
      .into_widget()
  }
}
