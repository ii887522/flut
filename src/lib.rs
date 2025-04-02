#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod app;
mod batches;
mod buffers;
mod engine;
pub mod models;
mod pipelines;
mod shaders;
mod string_slice;

pub use app::App;
pub use app::AppConfig;
pub use engine::Engine;
use sdl2::{event::Event, image::LoadSurface, surface::Surface};
use std::{mem, ptr, time::Instant};

pub fn run_app(mut app: impl App) {
  let app_config = app.get_config();
  let sdl = sdl2::init().unwrap();

  // Prevent SDL from creating an OpenGL context by itself
  sdl2::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");

  // Fix blurry UI on high DPI displays
  sdl2::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window(app_config.title, app_config.width, app_config.height)
    .allow_highdpi()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  if let Ok(favicon) = Surface::from_file("assets/favicon.png") {
    window.set_icon(favicon);
  }

  // Call window.show() as early as possible to minimize the perceived startup time
  window.show();

  let mut engine = Engine::new(window, app_config.prefer_dgpu);
  app.init(&mut engine);
  let mut event_pump = sdl.event_pump().unwrap();
  let mut prev = Instant::now();

  'running: loop {
    for event in event_pump.poll_iter() {
      if let Event::Quit { .. } = event {
        break 'running;
      }

      app.process_event(event);
    }

    let dt = prev.elapsed().as_secs_f32();
    prev = Instant::now();
    app.update(dt, &mut engine);
    engine.draw();
  }
}

const fn as_bytes<T>(from: &T) -> &[u8] {
  unsafe { &*ptr::slice_from_raw_parts(from as *const _ as *const _, mem::size_of::<T>()) }
}
