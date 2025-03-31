#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  flut::run_app()
    .title("Worm")
    .width(768u32)
    .height(768u32)
    .call();
}
