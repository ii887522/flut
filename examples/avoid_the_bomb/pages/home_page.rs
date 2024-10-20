use crate::i18n::I18N;
use flut::{
  models::{icon_name, HorizontalAlign, Lang, TextStyle, VerticalAlign},
  widgets::{
    button::{ButtonIcon, LabelStyle},
    router::Navigator,
    widget::*,
    Button, Column, ImageWidget, Row, Spacing, StatelessWidget, Text, Widget,
  },
};
use skia_safe::{Color, Rect};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(crate) struct HomePage<'a> {
  pub(crate) navigator: Arc<Mutex<Navigator<'a>>>,
}

impl<'a> StatelessWidget<'a> for HomePage<'a> {
  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let navigator_arc_1 = Arc::clone(&self.navigator);
    let navigator_arc_2 = Arc::clone(&self.navigator);
    let navigator_arc_3 = Arc::clone(&self.navigator);

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
            ImageWidget::new("assets/avoid_the_bomb/images/bomb.png")
              .size((64.0, 64.0))
              .call()
              .into_widget(),
            Spacing {
              width: 24.0,
              ..Default::default()
            }
            .into_widget(),
            Text::new()
              .text(I18N.with(|i18n| i18n.t("avoid_the_bomb").call()))
              .style(TextStyle {
                font_family: I18N.with(|i18n| i18n.get_default_font_family()),
                font_size: 64.0,
                color: Color::from_rgb(114, 114, 114),
                ..Default::default()
              })
              .call()
              .into_widget(),
          ])
          .call()
          .into_widget(),
        Spacing {
          height: 192.0,
          ..Default::default()
        }
        .into_widget(),
        Button {
          bg_color: Color::GREEN,
          icon: ButtonIcon::Icon {
            name: icon_name::SENTIMENT_VERY_DISSATISFIED,
            color: Color::BLACK,
          },
          label: I18N.with(|i18n| i18n.t("start_easy_game").call()),
          label_style: LabelStyle {
            font_family: I18N.with(|i18n| i18n.get_default_font_family()),
            ..Default::default()
          },
          size: I18N.with(|i18n| match i18n.get_current_lang() {
            Lang::Id => (480.0, 64.0),
            _ => (352.0, 64.0),
          }),
          on_mouse_up: Arc::new(Mutex::new(move || {
            let mut navigator = navigator_arc_1.lock().unwrap();
            navigator.go("/game?difficulty=easy");
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
          bg_color: Color::YELLOW,
          icon: ButtonIcon::Icon {
            name: icon_name::SENTIMENT_NEUTRAL,
            color: Color::BLACK,
          },
          label: I18N.with(|i18n| i18n.t("start_medium_game").call()),
          label_style: LabelStyle {
            font_family: I18N.with(|i18n| i18n.get_default_font_family()),
            ..Default::default()
          },
          size: I18N.with(|i18n| match i18n.get_current_lang() {
            Lang::Id => (480.0, 64.0),
            _ => (352.0, 64.0),
          }),
          on_mouse_up: Arc::new(Mutex::new(move || {
            let mut navigator = navigator_arc_2.lock().unwrap();
            navigator.go("/game?difficulty=medium");
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
          icon: ButtonIcon::Icon {
            name: icon_name::SENTIMENT_VERY_DISSATISFIED,
            color: Color::BLACK,
          },
          label: I18N.with(|i18n| i18n.t("start_hard_game").call()),
          label_style: LabelStyle {
            font_family: I18N.with(|i18n| i18n.get_default_font_family()),
            ..Default::default()
          },
          size: I18N.with(|i18n| match i18n.get_current_lang() {
            Lang::Id => (480.0, 64.0),
            _ => (352.0, 64.0),
          }),
          on_mouse_up: Arc::new(Mutex::new(move || {
            let mut navigator = navigator_arc_3.lock().unwrap();
            navigator.go("/game?difficulty=hard");
          })),
          ..Default::default()
        }
        .into_widget(),
      ])
      .call()
      .into_widget()
  }
}
