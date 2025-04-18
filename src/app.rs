use crate::{Engine, audio, engine::DrawableCaps};
use sdl2::{event::Event, image::LoadSurface, surface::Surface};
use std::{sync::mpsc, thread, time::Instant};

pub struct AppConfig {
  pub title: &'static str,
  pub width: u32,
  pub height: u32,
  pub prefer_dgpu: bool,
  pub drawable_caps: DrawableCaps,
}

impl Default for AppConfig {
  fn default() -> Self {
    AppConfig {
      title: "",
      width: 1024,
      height: 768,
      prefer_dgpu: false,
      drawable_caps: DrawableCaps::default(),
    }
  }
}

pub trait App {
  fn get_config(&self) -> AppConfig;
  fn init(&mut self, _engine: &mut Engine<'_>) {}
  fn process_event(&mut self, _event: Event) {}
  fn update(&mut self, _dt: f32, _engine: &mut Engine<'_>) {}
}

pub fn run(mut app: impl App) {
  let app_config = app.get_config();
  let sdl = sdl2::init().unwrap();

  let (audio_tx, audio_rx) = mpsc::channel();
  thread::spawn(|| audio::run(audio_rx));

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

  if let Ok(favicon) = Surface::from_file("assets/images/favicon.png") {
    window.set_icon(favicon);
  }

  // Call window.show() as early as possible to minimize the perceived startup time
  window.show();

  let mut engine = Engine::new(
    window,
    audio_tx,
    app_config.prefer_dgpu,
    app_config.drawable_caps,
  );

  app.init(&mut engine);

  const UPDATES_PER_SECOND: f32 = 150.0;
  const MAX_UPDATES_PER_FRAME: u32 = 5;

  let mut event_pump = sdl.event_pump().unwrap();
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
    let mut updates_remaining = MAX_UPDATES_PER_FRAME;

    while frame_time > 0.0 && updates_remaining > 0 {
      let dt = frame_time.min(1.0 / UPDATES_PER_SECOND);
      app.update(dt, &mut engine);
      frame_time -= dt;
      updates_remaining -= 1;
    }

    engine.draw();
  }
}
