use crate::i18n::I18N;
use atomic_refcell::AtomicRefCell;
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
use std::sync::Arc;

pub(crate) struct YouWonDialog<'a> {
  pub(crate) navigator: Arc<AtomicRefCell<Navigator<'a>>>,
  pub(crate) on_ok: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
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
      on_close: Arc::new(AtomicRefCell::new(move || {
        let mut navigator = navigator.borrow_mut();
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
