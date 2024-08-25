#![deny(elided_lifetimes_in_paths)]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod models;
mod pages;

use flut::{app, App};

fn main() {
  app::run(App {
    title: "Worm",
    size: (1280, 720),
    ..Default::default()
  });
}
