#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod game;

use flut::app::{self, ModelCapacities};
use game::Game;

#[cfg(not(feature = "reload"))]
use mimalloc::MiMalloc;

#[cfg(not(feature = "reload"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
  app::run(Game::new())
    .title("Worm")
    .size(worm_lib::WINDOW_SIZE)
    .favicon_path("assets/worm/images/favicon.png")
    .model_capacities(ModelCapacities {
      rect_capacity: 4096,
    })
    .call();
}
