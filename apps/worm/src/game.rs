use flut::{App, Context, Event};

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(
  dylib = "worm_lib",
  lib_dir = if cfg!(debug_assertions) { "apps/worm/lib/target/debug" } else { "apps/worm/lib/target/release" }
)]
mod lib {
  use flut::{Context, Event};
  pub use worm_lib::Game;

  hot_functions_from_file!("apps/worm/lib/src/lib.rs");
}

#[cfg(not(feature = "reload"))]
use worm_lib as lib;

pub(super) struct Game(lib::Game);

impl Game {
  #[inline]
  pub(super) fn new() -> Self {
    Self(lib::Game::new())
  }
}

impl App for Game {
  #[inline]
  fn init(&mut self, context: Context<'_>) {
    lib::init(&mut self.0, context);
  }

  #[inline]
  fn process_event(&mut self, event: Event) {
    lib::process_event(&mut self.0, event);
  }

  #[inline]
  fn update(&mut self, dt: f32, context: Context<'_>) {
    lib::update(&mut self.0, dt, context);
  }
}
