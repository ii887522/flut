use crate::renderer_ref::RendererRef;
use std::{borrow::Cow, time::Instant};

// Settings
const UPDATES_PER_SECOND: f32 = 160.0;
const MAX_FRAME_UPDATES: usize = 8;

pub struct AppLoop {
  title: Cow<'static, str>,
  show_fps: bool,
  prev: Instant,
  total_frame_time: f32,
  frame_count: usize,
}

impl AppLoop {
  #[must_use]
  #[inline]
  pub(super) fn new(title: Cow<'static, str>, show_fps: bool) -> Self {
    Self {
      title,
      show_fps,
      prev: Instant::now(),
      total_frame_time: 0.0,
      frame_count: 0,
    }
  }

  pub(super) fn update<OnUpdate: FnMut(f32, &mut RendererRef<'_>)>(
    &mut self,
    mut renderer: RendererRef<'_>,
    mut on_update: OnUpdate,
  ) {
    let frame_time = self.prev.elapsed();
    self.prev = Instant::now();
    let mut frame_time = frame_time.as_secs_f32();

    if self.show_fps {
      self.total_frame_time += frame_time;
      self.frame_count += 1;

      if self.total_frame_time >= 1.0 {
        let window = renderer.get_window();

        window.set_title(&format!(
          "{title} | {fps:.2} FPS",
          title = self.title,
          fps = 1.0 / (self.total_frame_time / self.frame_count as f32)
        ));

        self.total_frame_time = 0.0;
        self.frame_count = 0;
      }
    }

    let mut update_count = 0;

    while frame_time > 0.0 && update_count < MAX_FRAME_UPDATES {
      let dt = frame_time.min(1.0 / UPDATES_PER_SECOND);
      on_update(dt, &mut renderer);
      frame_time -= dt;
      update_count += 1;
    }
  }
}
