#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use flut::{App, AppConfig, Engine, models::Rect};
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

  fn init(&mut self, engine: &mut Engine<'_>) {
    engine.add_rect(Rect::new((384.0, 384.0), (243, 125, 121)));
  }

  fn process_event(&mut self, _event: Event) {}
}

fn main() {
  flut::run_app(WormApp);
}
