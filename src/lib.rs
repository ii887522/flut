#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

mod pipelines;
mod shaders;
mod string_slice;
mod vk_engine;

use optarg2chain::optarg_fn;
use sdl2::{event::Event, image::LoadSurface, surface::Surface};
use vk_engine::VkEngine;

#[optarg_fn(RunAppBuilder, call)]
pub fn run_app<'a>(
  #[optarg_default] title: &'a str,
  #[optarg(1024)] width: u32,
  #[optarg(768)] height: u32,
  #[optarg_default] prefer_dgpu: bool,
) {
  let sdl = sdl2::init().unwrap();

  // Prevent SDL from creating an OpenGL context by itself
  sdl2::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");

  // Fix blurry UI on high DPI displays
  sdl2::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

  let event_subsys = sdl.event().unwrap();
  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window(title, width, height)
    .allow_highdpi()
    .metal_view()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  if let Ok(favicon) = Surface::from_file("assets/favicon.png") {
    window.set_icon(favicon);
  }

  // Call window.show() as early as possible to minimize the perceived startup time
  window.show();

  let vk_engine = VkEngine::new(&window, prefer_dgpu);

  // Register `()` event for triggering acquire swapchain image at the next iteration in case the swapchain is recreated
  event_subsys.register_custom_event::<()>().unwrap();

  let event_sender = event_subsys.event_sender();
  let mut event_pump = sdl.event_pump().unwrap();

  for event in event_pump.wait_iter() {
    if let Event::Quit { .. } = event {
      break;
    }
  }
}
