use crate::{Context, audio, vk};
use flut_macro::warn;
use sdl2::{event::Event, image::LoadSurface, surface::Surface, ttf};
use std::{borrow::Cow, iter, sync::mpsc, thread, time::Instant};

pub struct DrawableCaps {
  pub rect_count: usize,
}

impl Default for DrawableCaps {
  fn default() -> Self {
    Self { rect_count: 3000 }
  }
}

pub struct Config {
  pub title: Cow<'static, str>,
  pub size: (u32, u32),
  pub favicon_path: Cow<'static, str>,
  pub font_path: Cow<'static, str>,
  pub drawable_caps: DrawableCaps,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      title: "".into(),
      size: (800, 600),
      favicon_path: "".into(),
      font_path: "".into(),
      drawable_caps: DrawableCaps::default(),
    }
  }
}

pub trait App {
  fn get_config(&self) -> Config {
    Default::default()
  }

  fn init(&mut self, _context: Context<'_>) {}
  fn process_event(&mut self, _event: Event) {}
  fn update(&mut self, _dt: f32, _context: Context<'_>) {}
}

pub fn run(mut app: impl App) {
  let sdl = sdl2::init().unwrap();
  let (audio_tx, audio_rx) = mpsc::channel();
  let audio_thread = thread::spawn(|| audio::run(audio_rx));

  // Prevent SDL from creating an OpenGL context by itself
  sdl2::hint::set("SDL_VIDEO_EXTERNAL_CONTEXT", "1");

  // Fix blurry UI on high DPI displays
  sdl2::hint::set("SDL_WINDOWS_DPI_AWARENESS", "permonitorv2");

  let vid_subsys = sdl.video().unwrap();
  let ttf = ttf::init().unwrap();
  let config = app.get_config();

  let mut window = vid_subsys
    .window(&config.title, config.size.0, config.size.1)
    .allow_highdpi()
    .hidden()
    .position_centered()
    .vulkan()
    .build()
    .unwrap();

  match Surface::from_file(&*config.favicon_path) {
    Ok(favicon) => window.set_icon(favicon),
    Err(err) => warn!("{err}"),
  }

  let mut renderer_result =
    vk::Renderer::new(ttf, &config.font_path, window.clone(), config.drawable_caps).finish();

  app.init(Context {
    renderer: &mut renderer_result,
    audio_tx: audio_tx.clone(),
    app_size: config.size,
  });

  window.show();
  let mut event_pump = sdl.event_pump().unwrap();
  let mut prev = Instant::now();

  'running: loop {
    for event in event_pump.poll_iter() {
      if let Event::Quit { .. } = event {
        break 'running;
      }

      app.process_event(event);
    }

    let frame_time = prev.elapsed().as_secs_f32();
    prev = Instant::now();

    iter::successors(
      Some((
        frame_time,
        frame_time,
        crate::consts::MAX_FRAME_UPDATE_COUNT,
      )),
      |&(_prev_time, next_time, update_left)| {
        if next_time > 0.0 && update_left > 0 {
          Some((
            next_time,
            (next_time - 1.0 / crate::consts::UPDATES_PER_SECOND).max(0.0),
            update_left - 1,
          ))
        } else {
          None
        }
      },
    )
    .skip(1)
    .map(|(prev_time, next_time, _)| prev_time - next_time)
    .for_each(|dt| {
      app.update(
        dt,
        Context {
          renderer: &mut renderer_result,
          audio_tx: audio_tx.clone(),
          app_size: config.size,
        },
      )
    });

    renderer_result = match renderer_result {
      Ok(renderer) => renderer.render(),
      Err(renderer) => renderer.finish(),
    };
  }

  match renderer_result {
    Ok(renderer) => renderer.drop(),
    Err(renderer) => renderer.drop(),
  }

  drop(audio_tx);
  audio_thread.join().unwrap();
}
