#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod game;

use flut::{app, models::ModelCapacities};
use game::Game;
use worm_lib::consts;

#[cfg(not(feature = "reload"))]
use mimalloc::MiMalloc;

#[cfg(not(feature = "reload"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
  app::run(Game::new())
    .title("Worm")
    .size(consts::WINDOW_SIZE)
    .favicon_path("assets/worm/images/favicon.png")
    .icon_font_path("assets/worm/fonts/MaterialSymbolsOutlined-Regular.ttf")
    .model_capacities(ModelCapacities {
      glyph_capacities: Box::new([4096, 128]),
      ..Default::default()
    })
    .call();
}
