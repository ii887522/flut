#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
#![deny(elided_lifetimes_in_paths)]

mod models;
mod pages;

use flut::{app, widgets::widget::*, App};
use pages::GamePage;

fn main() {
  app::run(App {
    title: "Worm",
    size: (660, 720),
    favicon_file_path: "assets/worm/images/favicon.png",
    use_audio: true,
    child: Some(GamePage::new().into_widget()),
  });
}
