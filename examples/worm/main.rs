#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mimalloc::MiMalloc;
use sdl3::{event::Event, image::LoadSurface, surface::Surface};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
  let sdl = sdl3::init().unwrap();
  sdl3::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");
  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window("Worm", 1600, 900)
    .high_pixel_density()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  if let Ok(favicon) = Surface::from_file("assets/images/favicon.png") {
    window.set_icon(favicon);
  }

  let mut event_pump = sdl.event_pump().unwrap();

  'running: loop {
    for event in event_pump.poll_iter() {
      if let Event::Quit { .. } = event {
        break 'running;
      }
    }
  }
}
