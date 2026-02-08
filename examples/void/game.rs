use flut::{
  app::App,
  models::{model_capacities::ModelCapacities, round_rect::RoundRect},
  utils,
};
use std::{iter, time::Instant};
use winit::{
  application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
  window::WindowId,
};

pub struct Game {
  app: Option<App>,
  prev: Instant,
  total_frame_time: f32,
  frame_count: usize,
  round_rect_ids: Box<[u32]>,
  round_rects: Box<[RoundRect]>,
}

impl Game {
  #[inline]
  pub(super) fn new() -> Self {
    Self {
      app: None,
      prev: Instant::now(),
      total_frame_time: 0.0,
      frame_count: 0,
      round_rect_ids: Box::new([]),
      round_rects: Box::new([]),
    }
  }
}

impl ApplicationHandler for Game {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.app.is_some() {
      return;
    }

    let mut app = App::new(
      event_loop,
      "Void",
      (1280_f64, 720_f64),
      ModelCapacities::default(),
    );

    let mut renderer = app.get_renderer();

    let round_rects = iter::repeat_with(|| RoundRect {
      position: (
        fastrand::f32() * 1280.0,
        fastrand::f32() * 720.0,
        fastrand::f32(),
      ),
      radius: fastrand::f32() * 32.0,
      size: (fastrand::f32() * 320.0, fastrand::f32() * 180.0),
      color: utils::pack_color(
        fastrand::u8(..=255),
        fastrand::u8(..=255),
        fastrand::u8(..=255),
        255,
      ),
    })
    .take(1000)
    .collect::<Box<_>>();

    let round_rect_ids = renderer.bulk_add_models(round_rects.clone());

    self.app = Some(app);
    self.prev = Instant::now();
    self.round_rect_ids = round_rect_ids;
    self.round_rects = round_rects;
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::RedrawRequested => {
        let Some(mut app) = self.app.take() else {
          return;
        };

        let frame_time = self.prev.elapsed();
        self.prev = Instant::now();
        self.total_frame_time += frame_time.as_secs_f32();
        self.frame_count += 1;

        let mut renderer = app.get_renderer();

        if self.total_frame_time >= 1.0 {
          let window = renderer.get_window();

          window.set_title(&format!(
            "Void: {:.2} FPS",
            1.0 / (self.total_frame_time / self.frame_count as f32)
          ));

          self.total_frame_time = 0.0;
          self.frame_count = 0;
        }

        renderer.bulk_update_models(
          &self.round_rect_ids,
          self
            .round_rects
            .iter()
            .map(|&round_rect| RoundRect {
              position: (
                fastrand::f32().mul_add(20.0, round_rect.position.0) - 10.0,
                fastrand::f32().mul_add(20.0, round_rect.position.1) - 10.0,
                round_rect.position.2,
              ),
              size: (
                fastrand::f32().mul_add(20.0, round_rect.size.0) - 10.0,
                fastrand::f32().mul_add(20.0, round_rect.size.1) - 10.0,
              ),
              ..round_rect
            })
            .collect(),
        );

        let app = app.render();
        app.request_redraw_if_visible();
        self.app = Some(app);
      }
      _ => (),
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(ref app) = self.app {
      app.request_redraw_if_visible();
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(app) = self.app.take() {
      app.drop();
    }
  }
}
