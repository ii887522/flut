#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sdl2::{event::Event, image::LoadSurface};
use vulkano::{
  Version, VulkanLibrary,
  instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
};

fn main() {
  let sdl = sdl2::init().unwrap();

  // Prevent SDL from creating an OpenGL context by itself
  sdl2::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");

  // Fix blurry UI on high DPI displays
  sdl2::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window("Worm", 1024, 768)
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
      application_name: Some("Worm".to_string()),
      application_version: Version {
        major: 0,
        minor: 1,
        patch: 0,
      },
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
