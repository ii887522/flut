use crate::i18n::I18N;
use flut::{
  models::{icon_name, TextStyle},
  widgets::{
    button::LabelStyle,
    dialog::{CloseButton, DialogHeader, OkButton, TitleStyle},
    router::Navigator,
    widget::*,
    Dialog, StatelessWidget, TextBlock, Widget,
  },
};
use skia_safe::{Color, Rect};
use std::{
  fmt::{self, Debug, Formatter},
  sync::{Arc, Mutex},
};

pub(crate) struct YouWonDialog<'a> {
  pub(crate) navigator: Arc<Mutex<Navigator<'a>>>,
  pub(crate) on_ok: Arc<Mutex<dyn FnMut() + 'a + Send>>,
}

impl Debug for YouWonDialog<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("GameOverDialog")
      .field("navigator", &self.navigator)
      .finish_non_exhaustive()
  }
}

impl<'a> StatelessWidget<'a> for YouWonDialog<'a> {
  fn get_size(&self) -> (f32, f32) {
    // (0.0, 0.0) so that this widget can be inserted in Column or Row or any other layout widget.
    // Size is ignored and this widget always cover the whole app
    (0.0, 0.0)
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let navigator = Arc::clone(&self.navigator);

    Dialog {
      color: Color::from_rgb(128, 255, 128),
      header: DialogHeader {
        icon: icon_name::SENTIMENT_VERY_SATISFIED,
        title: I18N.with(|i18n| i18n.t("you_won").call()),
        title_style: TitleStyle {
          font_family: I18N.with(|i18n| i18n.get_default_font_family()),
          ..Default::default()
        },
        ..Default::default()
      },
      has_ok: true,
      close_btn: CloseButton {
        icon: icon_name::HOME,
        label: I18N.with(|i18n| i18n.t("home").call()),
        label_style: LabelStyle {
          font_family: I18N.with(|i18n| i18n.get_default_font_family()),
          ..Default::default()
        },
        ..Default::default()
      },
      ok_btn: OkButton {
        icon: icon_name::RESTART_ALT,
        label: I18N.with(|i18n| i18n.t("try_again").call()),
        label_style: LabelStyle {
          font_family: I18N.with(|i18n| i18n.get_default_font_family()),
          ..Default::default()
        },
        ..Default::default()
      },
      on_close: Arc::new(Mutex::new(move || {
        let mut navigator = navigator.lock().unwrap();
        navigator.go("/");
      })),
      on_ok: Arc::clone(&self.on_ok),
      body: Some(
        TextBlock::new()
          .text(I18N.with(|i18n| i18n.t("you_won_desc").call()))
          .style(TextStyle {
            font_family: I18N.with(|i18n| i18n.get_default_font_family()),
            font_size: 24.0,
            ..Default::default()
          })
          .call()
          .into_widget(),
      ),
    }
    .into_widget()
  }
}
