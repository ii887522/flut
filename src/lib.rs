#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

use sdl2::{event::Event, image::LoadSurface};
use vulkano::{
  VulkanLibrary,
  instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
};

pub fn run_app(title: &str, width: u32, height: u32) {
  let sdl = sdl2::init().unwrap();

  // Prevent SDL from creating an OpenGL context by itself
  sdl2::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");

  // Fix blurry UI on high DPI displays
  sdl2::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window(title, width, height)
    .allow_highdpi()
    .metal_view()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  if let Ok(favicon) = sdl2::surface::Surface::from_file("assets/favicon.png") {
    window.set_icon(favicon);
  }

  // Call window.show() as early as possible to minimize the perceived startup time
  window.show();

  let vk_instance_exts =
    InstanceExtensions::from_iter(window.vulkan_instance_extensions().unwrap());

  let _vk_instance = Instance::new(
    VulkanLibrary::new().unwrap(),
    InstanceCreateInfo {
      enabled_extensions: vk_instance_exts,
      flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
      ..Default::default()
    },
  )
  .unwrap();

  let mut event_pump = sdl.event_pump().unwrap();

  for event in event_pump.wait_iter() {
    if let Event::Quit { .. } = event {
      break;
    }
  }
}
