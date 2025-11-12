use crate::renderer::{AnyRenderer, FinishError, Renderer};
use optarg2chain::optarg_fn;
use sdl3::{event::Event, image::LoadSurface, surface::Surface};
use std::borrow::Cow;

#[optarg_fn(RunBuilder, call)]
pub fn run(
  #[optarg_default] title: Cow<'static, str>,
  #[optarg((800, 600))] size: (u32, u32),
  #[optarg_default] favicon_path: Cow<'static, str>,
) {
  let sdl = sdl3::init().unwrap();
  sdl3::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");
  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window(&title, size.0, size.1)
    .high_pixel_density()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  if let Ok(favicon) = Surface::from_file(&*favicon_path) {
    window.set_icon(favicon);
  }

  let mut vk_renderer = Renderer::new(window.clone()).finish(window.clone());
  let mut event_pump = sdl.event_pump().unwrap();

  'running: loop {
    for event in event_pump.poll_iter() {
      if let Event::Quit { .. } = event {
        break 'running;
      }
    }

    vk_renderer = match vk_renderer {
      Ok(renderer) => match renderer.render() {
        AnyRenderer::Creating(renderer) => renderer.finish(window.clone()),
        AnyRenderer::Created(renderer) => Ok(renderer),
      },
      Err(FinishError::WindowMinimized(renderer)) => renderer.finish(window.clone()),
    };
  }

  match vk_renderer {
    Ok(renderer) => renderer.drop(),
    Err(FinishError::WindowMinimized(renderer)) => (*renderer).drop(),
  }
}
