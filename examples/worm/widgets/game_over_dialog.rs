use crate::i18n::I18N;
use flut::{
  models::{icon_name, Value},
  widgets::{router::Navigator, widget::*, Dialog, StatelessWidget, TextBlock, Widget},
};
use skia_safe::{Color, Rect};
use std::{
  fmt::{self, Debug, Formatter},
  sync::{Arc, Mutex},
};

pub(crate) struct GameOverDialog<'a> {
  pub(crate) navigator: Arc<Mutex<Navigator<'a>>>,
  pub(crate) score: u32,
  pub(crate) on_ok: Arc<Mutex<dyn FnMut() + 'a + Send>>,
}

impl Debug for GameOverDialog<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("GameOverDialog")
      .field("navigator", &self.navigator)
      .field("score", &self.score)
      .finish_non_exhaustive()
  }
}

impl<'a> StatelessWidget<'a> for GameOverDialog<'a> {
  fn get_size(&self) -> (f32, f32) {
    // (0.0, 0.0) so that this widget can be inserted in Column or Row or any other layout widget.
    // Size is ignored and this widget always cover the whole app
    (0.0, 0.0)
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let navigator = Arc::clone(&self.navigator);

    Dialog {
      color: Color::from_rgb(255, 128, 128),
      header_icon: icon_name::SKULL,
      header_title: I18N.with(|i18n| i18n.t("you_died").call()),
      header_title_font_family: I18N.with(|i18n| i18n.get_default_font_family()),
      has_ok: true,
      close_icon: icon_name::SENTIMENT_DISSATISFIED,
      close_label: I18N.with(|i18n| i18n.t("give_up").call()),
      close_label_font_family: I18N.with(|i18n| i18n.get_default_font_family()),
      ok_icon: icon_name::RESTART_ALT,
      ok_label: I18N.with(|i18n| i18n.t("try_again").call()),
      ok_label_font_family: I18N.with(|i18n| i18n.get_default_font_family()),
      on_close: Arc::new(Mutex::new(move || {
        let mut navigator = navigator.lock().unwrap();
        navigator.go("/");
      })),
      on_ok: Arc::clone(&self.on_ok),
      body: Some(
        TextBlock::new()
          .text(I18N.with(|i18n| {
            i18n
              .t("you_died_desc")
              .message_args(&[("score", Value::Uint(self.score))][..])
              .call()
          }))
          .font_family(I18N.with(|i18n| i18n.get_default_font_family()))
          .font_size(24.0)
          .call()
          .into_widget(),
      ),
      ..Default::default()
    }
    .into_widget()
  }
}
