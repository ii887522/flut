use crate::{
  pipelines::rect_pipeline::Rect,
  renderers::{
    Renderer, RendererRef,
    renderer::{AnyRenderer, FinishError},
  },
};
use optarg2chain::optarg_fn;
use sdl3::{event::Event, image::LoadSurface, surface::Surface, video::WindowPos};
use std::{borrow::Cow, mem, time::Instant};

pub trait App {
  fn init(&mut self, _renderer: RendererRef<'_>) {}
  fn process_event(&mut self, _event: Event) {}
  fn update(&mut self, _dt: f32, _renderer: RendererRef<'_>) {}
}

pub struct ModelCapacities {
  pub rect_capacity: usize,
}

impl Default for ModelCapacities {
  #[inline]
  fn default() -> Self {
    Self {
      rect_capacity: 1024,
    }
  }
}

impl ModelCapacities {
  #[inline]
  pub(super) const fn calc_total_size(&self) -> usize {
    self.rect_capacity * mem::size_of::<Rect>()
  }
}

#[optarg_fn(RunBuilder, call)]
pub fn run<A: App>(
  mut app: A,
  #[optarg_default] title: Cow<'static, str>,
  #[optarg((800, 600))] size: (u32, u32),
  #[optarg_default] favicon_path: Cow<'static, str>,
  #[optarg_default] model_capacities: ModelCapacities,
) {
  let sdl = sdl3::init().unwrap();
  sdl3::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");
  let vid_subsys = sdl.video().unwrap();

  let mut window = vid_subsys
    .window(&title, size.0, size.1)
    .high_pixel_density()
    .position_centered()
    .vulkan()
    .hidden()
    .build()
    .unwrap();

  match window.get_display() {
    Ok(window_display) => match window_display.get_content_scale() {
      Ok(display_content_scale) => match window.set_size(
        (size.0 as f32 * display_content_scale) as _,
        (size.1 as f32 * display_content_scale) as _,
      ) {
        Ok(_) => {
          if !window.set_position(WindowPos::Centered, WindowPos::Centered) {
            println!("Failed to re-center the window");
          }
        }
        Err(err) => println!("Failed to calibrate window size: {err}"),
      },
      Err(err) => println!("Failed to get display content scale: {err}"),
    },
    Err(err) => println!("Failed to get window display: {err}"),
  }

  if let Ok(favicon) = Surface::from_file(&*favicon_path) {
    window.set_icon(favicon);
  }

  let mut vk_renderer = Renderer::new(window.clone(), model_capacities).finish(window.clone());
  app.init(RendererRef::new(&mut vk_renderer));
  window.show();
  let mut event_pump = sdl.event_pump().unwrap();
  const UPDATES_PER_SECOND: f32 = 120.0;
  const MAX_UPDATES_PER_FRAME: usize = 8;
  let mut prev = Instant::now();

  'running: loop {
    for event in event_pump.poll_iter() {
      if let Event::Quit { .. } = event {
        break 'running;
      }

      app.process_event(event);
    }

    let mut frame_time = prev.elapsed().as_secs_f32();
    prev = Instant::now();
    let mut update_count = 0;

    while frame_time > 0.0 && update_count < MAX_UPDATES_PER_FRAME {
      let dt = frame_time.min(1.0 / UPDATES_PER_SECOND);
      app.update(dt, RendererRef::new(&mut vk_renderer));
      frame_time -= dt;
      update_count += 1;
    }

    vk_renderer = match vk_renderer {
      Ok(renderer) => match renderer.render(window.clone()) {
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
