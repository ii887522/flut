#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! A Snake-like game implementation where the player controls a worm that grows by eating food
//! while avoiding walls and itself.

mod consts;
mod models;
mod worm_game;

use flut::app;
use mimalloc::MiMalloc;
use worm_game::WormGame;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
  app::run(WormGame::new());
}
