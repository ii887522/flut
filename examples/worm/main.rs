#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use flut::{App, AppConfig};
use sdl2::event::Event;

struct WormApp;

impl App for WormApp {
  fn get_config(&self) -> AppConfig {
    AppConfig {
      title: "Worm",
      width: 768,
      height: 768,
      ..Default::default()
    }
  }

  fn process_event(&mut self, _event: Event) {}
  fn update(&mut self, _dt: f32) {}
  fn draw(&self) {}
}

fn main() {
  flut::run_app(WormApp);
}
