use super::{Container, Glass};
use crate::{
  Engine, Transition,
  models::{Rect, RoundRect},
};
use sdl2::event::Event;
use std::sync::atomic::Ordering;

const POP_UP_DURATION: f32 = 0.125;
const DIALOG_SIZE: (u32, u32) = (512, 256);

#[derive(Clone, Copy)]
pub struct Dialog {
  glass: Glass,
  container: Container,
}

impl Default for Dialog {
  fn default() -> Self {
    Self::new()
  }
}

impl Dialog {
  pub fn new() -> Self {
    let app_size = (
      crate::APP_SIZE.0.load(Ordering::Relaxed),
      crate::APP_SIZE.1.load(Ordering::Relaxed),
    );

    Self {
      glass: Glass {
        size: (app_size.0 as _, app_size.1 as _),
        alpha: Transition::new(0.0, 128.0, POP_UP_DURATION),
        drawable_id: u16::MAX,
      },
      container: Container {
        position: (
          Transition::new(
            (app_size.0 >> 1) as _,
            ((app_size.0 - DIALOG_SIZE.0) >> 1) as _,
            POP_UP_DURATION,
          ),
          Transition::new(
            (app_size.1 >> 1) as _,
            ((app_size.1 - DIALOG_SIZE.1) >> 1) as _,
            POP_UP_DURATION,
          ),
        ),
        size: (
          Transition::new(0.0, DIALOG_SIZE.0 as _, POP_UP_DURATION),
          Transition::new(0.0, DIALOG_SIZE.1 as _, POP_UP_DURATION),
        ),
        color: (255, 0, 0, 255),
        border_radius: Transition::new(0.0, 8.0, POP_UP_DURATION),
        drawable_id: u16::MAX,
      },
    }
  }

  pub fn init(&mut self, engine: &mut Engine<'_>) {
    self.glass.drawable_id = engine.add_rect(Rect::from(self.glass));
    self.container.drawable_id = engine.add_round_rect(RoundRect::from(self.container));
  }

  pub fn process_event(&mut self, event: Event) {
    match event {
      Event::MouseButtonDown {
        mouse_btn, x, y, ..
      } => {
        // todo
      }
      Event::MouseButtonUp {
        mouse_btn, x, y, ..
      } => {
        // todo
      }
      _ => {}
    }
  }

  pub fn update(&mut self, dt: f32, engine: &mut Engine<'_>) {
    self.glass.update(dt);
    self.container.update(dt);
    engine.update_rect(self.glass.drawable_id, Rect::from(self.glass));
    engine.update_round_rect(self.container.drawable_id, RoundRect::from(self.container));
  }
}
