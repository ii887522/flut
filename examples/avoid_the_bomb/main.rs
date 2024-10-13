#![deny(elided_lifetimes_in_paths)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod i18n;
mod models;
mod pages;
mod widgets;

use flut::{
  app,
  widgets::{widget::*, Router},
  App,
};
use i18n::I18N;
use models::Difficulty;
use pages::{GamePage, HomePage};
use std::{collections::HashMap, sync::Arc};

fn main() {
  app::run(App {
    favicon_file_path: "assets/avoid_the_bomb/images/bomb.png",
    title: &I18N.with(|i18n| i18n.t("avoid_the_bomb").call()),
    size: (660, 660),
    use_audio: true,
    child: Some(
      Router::new("/", |navigator| {
        HashMap::from_iter([
          ("/", {
            let navigator = Arc::clone(&navigator);

            Box::new(move |_qs_params: HashMap<&str, &str>| {
              HomePage {
                navigator: Arc::clone(&navigator),
              }
              .into_widget()
            }) as _
          }),
          ("/game", {
            let navigator = Arc::clone(&navigator);

            Box::new(move |qs_params: HashMap<&str, &str>| {
              GamePage {
                navigator: Arc::clone(&navigator),
                difficulty: Difficulty::from(*qs_params.get("difficulty").unwrap_or(&"")),
              }
              .into_widget()
            }) as _
          }),
        ])
      })
      .into_widget(),
    ),
  })
}
