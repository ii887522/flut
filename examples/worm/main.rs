#![deny(elided_lifetimes_in_paths)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod models;
mod pages;

use flut::{
  app,
  widgets::{widget::*, Router},
  App,
};
use pages::{GamePage, HomePage};
use std::{collections::HashMap, sync::Arc};

fn main() {
  app::run(App {
    title: "Worm",
    size: (660, 720),
    use_audio: true,
    child: Some(
      Router::new("/".to_string(), |navigator| {
        HashMap::from_iter([
          (
            "/".to_string(),
            HomePage {
              navigator: Arc::clone(&navigator),
            }
            .into_widget(),
          ),
          ("/game".to_string(), GamePage { navigator }.into_widget()),
        ])
      })
      .into_widget(),
    ),
  });
}
