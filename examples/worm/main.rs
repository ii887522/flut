#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use flut::{App, app};
use mimalloc::MiMalloc;
use sdl2::event::Event;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
  app::run(WormGame);
}

struct WormGame;

impl App for WormGame {
  fn get_config(&self) -> app::Config {
    app::Config {
      title: "Worm".into(),
      size: (900, 900),
      favicon_path: "assets/worm/favicon.png".into(),
      ..Default::default()
    }
  }

  fn init(&mut self) {}
  fn process_event(&mut self, _event: Event) {}
  fn update(&mut self, _dt: f32) {}
}
