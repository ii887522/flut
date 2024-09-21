#![deny(elided_lifetimes_in_paths)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod models;
mod pages;

use flut::{app, widgets::widget::*, App};
use pages::{GamePage, HomePage};

fn main() {
  app::run(App {
    title: "Worm",
    size: (660, 720),
    use_audio: true,
    child: Some(HomePage.into_widget()),
  });
}
