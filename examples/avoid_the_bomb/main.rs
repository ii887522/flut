#![deny(elided_lifetimes_in_paths)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod i18n;
mod models;
mod pages;

use flut::{
  app,
  widgets::{widget::*, Router},
  App,
};
use i18n::I18N;
use pages::GamePage;
use std::collections::HashMap;

fn main() {
  app::run(App {
    favicon_file_path: "assets/avoid_the_bomb/images/bomb.png",
    title: &I18N.with(|i18n| i18n.t("avoid_the_bomb").call()),
    size: (660, 660),
    use_audio: false,
    child: Some(
      Router::new("/game", |navigator| {
        HashMap::from_iter([("/game", GamePage { navigator }.into_widget())])
      })
      .into_widget(),
    ),
  })
}
