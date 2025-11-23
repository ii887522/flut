use flut::{App, Event, renderers::RendererRef};

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(
  dylib = "worm_lib",
  lib_dir = if cfg!(debug_assertions) { "apps/worm/lib/target/debug" } else { "apps/worm/lib/target/release" }
)]
mod lib {
  use flut::{Event, renderers::RendererRef};
  pub use worm_lib::Game;

  hot_functions_from_file!("apps/worm/lib/src/lib.rs");
}

#[cfg(not(feature = "reload"))]
use worm_lib as lib;

pub(super) struct Game(lib::Game);

impl Game {
  #[inline]
  pub(super) const fn new() -> Self {
    Self(lib::Game::new())
  }
}

impl App for Game {
  #[inline]
  fn init(&mut self, renderer: RendererRef<'_>) {
    lib::init(&mut self.0, renderer);
  }

  #[inline]
  fn process_event(&mut self, event: Event) {
    lib::process_event(&mut self.0, event);
  }

  #[inline]
  fn update(&mut self, dt: f32, renderer: RendererRef<'_>) {
    lib::update(&mut self.0, dt, renderer);
  }
}
