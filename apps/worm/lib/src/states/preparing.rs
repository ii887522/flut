use crate::{
  Countdown, consts,
  states::{Playing, State},
};
use flut::{
  Context,
  models::{Align, Text},
};

// Settings
const MAX_COUNTDOWN: u32 = 3;
const COUNTDOWN_MIN_FONT_SIZE: u16 = 64;
const COUNTDOWN_MAX_FONT_SIZE: u16 = 128;
const COUNTDOWN_INTERVAL: f32 = 0.75;
const COUNTDOWN_COLOR: (f32, f32, f32) = (1.0, 1.0, 1.0);
const COUNTDOWN_MAX_ALPHA: f32 = 0.5;

// Computed settings
const COUNTDOWN_POSITION: (f32, f32) = (
  consts::WINDOW_SIZE.0 as f32 * 0.5,
  consts::WINDOW_SIZE.1 as f32 * 0.5 + COUNTDOWN_MAX_FONT_SIZE as f32 * 0.5,
);

#[derive(Default)]
pub(crate) struct Preparing {
  countdown: Option<Countdown>,
}

impl Preparing {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self { countdown: None }
  }

  pub(crate) fn init(mut self, context: &mut Context<'_>) -> Self {
    let countdown_render_id = context.renderer.add_text(Text {
      position: COUNTDOWN_POSITION,
      color: (1.0, 1.0, 1.0, 0.5),
      font_size: COUNTDOWN_MIN_FONT_SIZE,
      align: Align::Center,
      text: MAX_COUNTDOWN.to_string().into(),
    });

    self.countdown = Some(Countdown {
      countdown: MAX_COUNTDOWN,
      render_id: countdown_render_id,
      accum: 0.0,
    });

    self
  }

  pub(crate) fn update(mut self, dt: f32, context: &mut Context<'_>) -> State {
    let countdown = self.countdown.as_mut().unwrap();
    countdown.accum += dt;

    if countdown.accum < COUNTDOWN_INTERVAL {
      let t = countdown.accum / COUNTDOWN_INTERVAL;

      context.renderer.update_text(
        countdown.render_id,
        Text {
          position: COUNTDOWN_POSITION,
          color: (
            COUNTDOWN_COLOR.0,
            COUNTDOWN_COLOR.1,
            COUNTDOWN_COLOR.2,
            COUNTDOWN_MAX_ALPHA * (1.0 - t),
          ),
          font_size: COUNTDOWN_MIN_FONT_SIZE
            + (t * (COUNTDOWN_MAX_FONT_SIZE - COUNTDOWN_MIN_FONT_SIZE) as f32) as u16,
          align: Align::Center,
          text: countdown.countdown.to_string().into(),
        },
      );

      return State::Preparing(self);
    }

    countdown.accum -= COUNTDOWN_INTERVAL;
    countdown.countdown -= 1;

    if countdown.countdown > 0 {
      context.renderer.update_text(
        countdown.render_id,
        Text {
          position: COUNTDOWN_POSITION,
          color: (1.0, 1.0, 1.0, 0.5),
          font_size: COUNTDOWN_MIN_FONT_SIZE,
          align: Align::Center,
          text: countdown.countdown.to_string().into(),
        },
      );

      return State::Preparing(self);
    }

    context.renderer.remove_text(countdown.render_id);
    State::Playing(Playing::new(context))
  }
}
