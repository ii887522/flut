use super::{Container, Glass};
use crate::{
  Engine, Transition,
  models::{Icon, IconName, Rect, RoundRect},
};
use sdl2::{event::Event, mouse::MouseButton};
use std::sync::atomic::Ordering;

// Config
const SCALING_UP_DURATION: f32 = 0.125;
const DIALOG_SIZE: (u32, u32) = (512, 256);
const VIBRATE_DURATION: f32 = 0.05;
const MIN_VIBRATE_SCALE: f32 = 0.9;
const DIALOG_BORDER_RADIUS: f32 = 10.0;

#[derive(Clone, Copy)]
enum State {
  ScalingUp,
  Idle,
  ScalingDown,
}

#[derive(Clone, Copy)]
pub struct Dialog {
  glass: Glass,
  container: Container,
  icon: Icon,
  is_mouse_down_outside: bool,
  state: State,
}

impl Dialog {
  pub fn new(bg_color: (u8, u8, u8, u8), color: (u8, u8, u8, u8), icon: IconName) -> Self {
    let app_size = (
      crate::APP_SIZE.0.load(Ordering::Relaxed),
      crate::APP_SIZE.1.load(Ordering::Relaxed),
    );

    let to_position = Self::get_position();

    Self {
      glass: Glass {
        size: (app_size.0 as _, app_size.1 as _),
        alpha: Transition::new(0.0, 128.0, SCALING_UP_DURATION),
        drawable_id: u16::MAX,
      },
      container: Container {
        position: (
          Transition::new(
            (app_size.0 >> 1) as _,
            to_position.0 as _,
            SCALING_UP_DURATION,
          ),
          Transition::new(
            (app_size.1 >> 1) as _,
            to_position.1 as _,
            SCALING_UP_DURATION,
          ),
        ),
        size: (
          Transition::new(0.0, DIALOG_SIZE.0 as _, SCALING_UP_DURATION),
          Transition::new(0.0, DIALOG_SIZE.1 as _, SCALING_UP_DURATION),
        ),
        bg_color,
        border_radius: Transition::new(0.0, DIALOG_BORDER_RADIUS, SCALING_UP_DURATION),
        drawable_id: u16::MAX,
      },
      icon: Icon::new((to_position.0 as _, to_position.1 as _), color, icon),
      is_mouse_down_outside: false,
      state: State::ScalingUp,
    }
  }

  fn get_position() -> (u32, u32) {
    let app_size = (
      crate::APP_SIZE.0.load(Ordering::Relaxed),
      crate::APP_SIZE.1.load(Ordering::Relaxed),
    );

    (
      (app_size.0 - DIALOG_SIZE.0) >> 1,
      (app_size.1 - DIALOG_SIZE.1) >> 1,
    )
  }

  fn is_mouse_on_this(mouse_position: (i32, i32)) -> bool {
    let position = Self::get_position();

    mouse_position.0 >= position.0 as _
      && mouse_position.0 < (position.0 + DIALOG_SIZE.0) as _
      && mouse_position.1 >= position.1 as _
      && mouse_position.1 < (position.1 + DIALOG_SIZE.1) as _
  }

  pub fn init(&mut self, engine: &mut Engine<'_>) {
    self.glass.drawable_id = engine.add_rect(Rect::from(self.glass));
    self.container.drawable_id = engine.add_round_rect(RoundRect::from(self.container));
    engine.add_icon(self.icon);
  }

  pub fn process_event(&mut self, event: &Event) {
    match event {
      Event::MouseButtonDown {
        mouse_btn,
        x: mouse_x,
        y: mouse_y,
        ..
      } => {
        if *mouse_btn == MouseButton::Left && !Self::is_mouse_on_this((*mouse_x, *mouse_y)) {
          self.is_mouse_down_outside = true;
        }
      }
      Event::MouseButtonUp {
        mouse_btn,
        x: mouse_x,
        y: mouse_y,
        ..
      } => {
        if *mouse_btn != MouseButton::Left {
          return;
        }

        if !self.is_mouse_down_outside
          || !matches!(self.state, State::Idle)
          || Self::is_mouse_on_this((*mouse_x, *mouse_y))
        {
          self.is_mouse_down_outside = false;
          return;
        }

        self.is_mouse_down_outside = false;
        let from_position = Self::get_position();

        let app_size = (
          crate::APP_SIZE.0.load(Ordering::Relaxed),
          crate::APP_SIZE.1.load(Ordering::Relaxed),
        );

        self.container = Container {
          position: (
            Transition::new(
              from_position.0 as _,
              crate::map(
                MIN_VIBRATE_SCALE,
                0.0,
                1.0,
                (app_size.0 >> 1) as _,
                from_position.0 as _,
              ),
              VIBRATE_DURATION,
            ),
            Transition::new(
              from_position.1 as _,
              crate::map(
                MIN_VIBRATE_SCALE,
                0.0,
                1.0,
                (app_size.1 >> 1) as _,
                from_position.1 as _,
              ),
              VIBRATE_DURATION,
            ),
          ),
          size: (
            Transition::new(
              DIALOG_SIZE.0 as _,
              DIALOG_SIZE.0 as f32 * MIN_VIBRATE_SCALE,
              VIBRATE_DURATION,
            ),
            Transition::new(
              DIALOG_SIZE.1 as _,
              DIALOG_SIZE.1 as f32 * MIN_VIBRATE_SCALE,
              VIBRATE_DURATION,
            ),
          ),
          bg_color: self.container.bg_color,
          border_radius: Transition::new(
            DIALOG_BORDER_RADIUS,
            crate::map(MIN_VIBRATE_SCALE, 0.0, 1.0, 0.0, DIALOG_BORDER_RADIUS),
            VIBRATE_DURATION,
          ),
          drawable_id: self.container.drawable_id,
        };

        self.state = State::ScalingDown;
      }
      _ => {}
    }
  }

  pub fn update(&mut self, dt: f32, engine: &mut Engine<'_>) {
    let prev_state = self.state;

    if self.glass.update(dt) & self.container.update(dt) {
      match prev_state {
        State::ScalingUp => self.state = State::Idle,
        State::ScalingDown => {
          let from_position = Self::get_position();

          let app_size = (
            crate::APP_SIZE.0.load(Ordering::Relaxed),
            crate::APP_SIZE.1.load(Ordering::Relaxed),
          );

          self.container = Container {
            position: (
              Transition::new(
                crate::map(
                  MIN_VIBRATE_SCALE,
                  0.0,
                  1.0,
                  (app_size.0 >> 1) as _,
                  from_position.0 as _,
                ),
                from_position.0 as _,
                VIBRATE_DURATION,
              ),
              Transition::new(
                crate::map(
                  MIN_VIBRATE_SCALE,
                  0.0,
                  1.0,
                  (app_size.1 >> 1) as _,
                  from_position.1 as _,
                ),
                from_position.1 as _,
                VIBRATE_DURATION,
              ),
            ),
            size: (
              Transition::new(
                DIALOG_SIZE.0 as f32 * MIN_VIBRATE_SCALE,
                DIALOG_SIZE.0 as _,
                VIBRATE_DURATION,
              ),
              Transition::new(
                DIALOG_SIZE.1 as f32 * MIN_VIBRATE_SCALE,
                DIALOG_SIZE.1 as _,
                VIBRATE_DURATION,
              ),
            ),
            bg_color: self.container.bg_color,
            border_radius: Transition::new(
              crate::map(MIN_VIBRATE_SCALE, 0.0, 1.0, 0.0, DIALOG_BORDER_RADIUS),
              DIALOG_BORDER_RADIUS,
              VIBRATE_DURATION,
            ),
            drawable_id: self.container.drawable_id,
          };

          self.state = State::ScalingUp;
        }
        _ => {}
      }
    }

    if !matches!(prev_state, State::Idle) {
      engine.update_rect(self.glass.drawable_id, Rect::from(self.glass));
      engine.update_round_rect(self.container.drawable_id, RoundRect::from(self.container));
    }
  }
}
