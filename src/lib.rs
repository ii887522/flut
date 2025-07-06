#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

mod models;
mod vk;

use std::{mem, slice};

use flut_macro::warn;
use sdl2::{event::Event, image::LoadSurface, surface::Surface};

pub fn run_app(title: &str, size: (u32, u32), favicon_path: &str) {
  let sdl = sdl2::init().unwrap();

  // Prevent SDL from creating an OpenGL context by itself
  sdl2::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");

  // Fix blurry UI on high DPI displays
  sdl2::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window(title, size.0, size.1)
    .allow_highdpi()
    .hidden()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  match Surface::from_file(favicon_path) {
    Ok(favicon) => window.set_icon(favicon),
    Err(err) => warn!("{err}"),
  }

  let mut renderer_result = vk::Renderer::new(window.clone()).finish();
  window.show();
  let mut event_pump = sdl.event_pump().unwrap();

  'running: loop {
    for event in event_pump.poll_iter() {
      if let Event::Quit { .. } = event {
        break 'running;
      }
    }

    renderer_result = match renderer_result {
      Ok(renderer) => renderer.render(),
      Err(renderer) => renderer.finish(),
    };
  }

  match renderer_result {
    Ok(renderer) => renderer.drop(),
    Err(renderer) => renderer.drop(),
  }
}

const fn as_bytes<T>(item: &T) -> &[u8] {
  unsafe { slice::from_raw_parts(item as *const _ as *const _, mem::size_of::<T>()) }
}
