use crate::i18n::I18N;
use atomic_refcell::AtomicRefCell;
use flut::{
  models::{icon_name, HorizontalAlign, Lang, TextStyle, VerticalAlign},
  widgets::{
    button::{ButtonIcon, LabelStyle},
    router::Navigator,
    select::SelectOption,
    widget::*,
    Button, Column, ImageWidget, Row, Select, Spacing, StatelessWidget, Text, Widget,
  },
};
use skia_safe::{Color, Rect};
use std::{process, sync::Arc};

pub(crate) struct HomePage<'a> {
  pub(crate) navigator: Arc<AtomicRefCell<Navigator<'a>>>,
}

impl<'a> StatelessWidget<'a> for HomePage<'a> {
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
            ImageWidget::new("assets/worm/images/favicon.png")
              .size((64.0, 64.0))
              .call()
              .into_widget(),
            Spacing {
              width: 24.0,
              ..Default::default()
            }
            .into_widget(),
            Text::new()
              .text(I18N.with(|i18n| i18n.t("worm").call()))
              .style(TextStyle {
                font_family: I18N.with(|i18n| i18n.get_default_font_family()),
                color: Color::from_rgb(243, 125, 121),
                font_size: 64.0,
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
        Select {
          size: (320.0, 64.0),
          is_elevated: false,
          bg_color: Color::from_rgb(224, 224, 224),
          option_bg_color: Color::from_rgb(192, 192, 192),
          options: Arc::new(vec![
            SelectOption::new()
              .icon(ButtonIcon::Image {
                file_path: "assets/worm/images/uk.png",
                tint: Color::WHITE,
              })
              .label("English")
              .label_font_family(Lang::En.get_default_font_family())
              .call(),
            SelectOption::new()
              .icon(ButtonIcon::Image {
                file_path: "assets/worm/images/id.png",
                tint: Color::WHITE,
              })
              .label("Indonesia")
              .label_font_family(Lang::Id.get_default_font_family())
              .call(),
            SelectOption::new()
              .icon(ButtonIcon::Image {
                file_path: "assets/worm/images/cn.png",
                tint: Color::WHITE,
              })
              .label("简体中文")
              .label_font_family(Lang::ZhCn.get_default_font_family())
              .call(),
            SelectOption::new()
              .icon(ButtonIcon::Image {
                file_path: "assets/worm/images/tw.png",
                tint: Color::WHITE,
              })
              .label("繁體中文")
              .label_font_family(Lang::ZhTw.get_default_font_family())
              .call(),
          ]),
          ..Default::default()
        }
        .into_widget(),
        Spacing {
          height: 64.0,
          ..Default::default()
        }
        .into_widget(),
        Button {
          bg_color: Color::GREEN,
          icon: ButtonIcon::Icon {
            name: icon_name::PLAY_ARROW,
            color: Color::BLACK,
          },
          label: I18N.with(|i18n| i18n.t("start_game").call()),
          label_style: LabelStyle {
            font_family: I18N.with(|i18n| i18n.get_default_font_family()),
            ..Default::default()
          },
          size: I18N.with(|i18n| match i18n.get_current_lang() {
            Lang::Id => (384.0, 64.0),
            _ => (256.0, 64.0),
          }),
          on_mouse_up: Arc::new(AtomicRefCell::new(move || {
            let mut navigator = navigator.borrow_mut();
            navigator.go("/game");
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
            name: icon_name::LOGOUT,
            color: Color::BLACK,
          },
          label: I18N.with(|i18n| i18n.t("exit_game").call()),
          label_style: LabelStyle {
            font_family: I18N.with(|i18n| i18n.get_default_font_family()),
            ..Default::default()
          },
          size: I18N.with(|i18n| match i18n.get_current_lang() {
            Lang::Id => (384.0, 64.0),
            _ => (256.0, 64.0),
          }),
          on_mouse_up: Arc::new(AtomicRefCell::new(|| process::exit(0))),
          ..Default::default()
        }
        .into_widget(),
      ])
      .call()
      .into_widget()
  }
}
