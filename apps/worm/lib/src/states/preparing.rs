use crate::{
  Countdown, consts,
  states::{Playing, State},
};
use flut::{
  Context,
  models::{Align, Text},
};

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
      position: consts::COUNTDOWN_POSITION,
      color: (1.0, 1.0, 1.0, 0.5),
      font_size: consts::COUNTDOWN_MIN_FONT_SIZE,
      align: Align::Center,
      text: consts::MAX_COUNTDOWN.to_string().into(),
    });

    self.countdown = Some(Countdown {
      countdown: consts::MAX_COUNTDOWN,
      render_id: countdown_render_id,
      accum: 0.0,
    });

    self
  }

  pub(crate) fn update(mut self, dt: f32, context: &mut Context<'_>) -> State {
    let countdown = self.countdown.as_mut().unwrap();
    countdown.accum += dt;

    if countdown.accum < consts::COUNTDOWN_INTERVAL {
      let t = countdown.accum / consts::COUNTDOWN_INTERVAL;

      context.renderer.update_text(
        countdown.render_id,
        Text {
          position: consts::COUNTDOWN_POSITION,
          color: (
            consts::COUNTDOWN_COLOR.0,
            consts::COUNTDOWN_COLOR.1,
            consts::COUNTDOWN_COLOR.2,
            consts::COUNTDOWN_MAX_ALPHA * (1.0 - t),
          ),
          font_size: consts::COUNTDOWN_MIN_FONT_SIZE
            + (t * (consts::COUNTDOWN_MAX_FONT_SIZE - consts::COUNTDOWN_MIN_FONT_SIZE) as f32)
              as u16,
          align: Align::Center,
          text: countdown.countdown.to_string().into(),
        },
      );

      return State::Preparing(self);
    }

    countdown.accum -= consts::COUNTDOWN_INTERVAL;
    countdown.countdown -= 1;

    if countdown.countdown > 0 {
      context.renderer.update_text(
        countdown.render_id,
        Text {
          position: consts::COUNTDOWN_POSITION,
          color: (1.0, 1.0, 1.0, 0.5),
          font_size: consts::COUNTDOWN_MIN_FONT_SIZE,
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
