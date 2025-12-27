use crate::{State, consts, states::ShowingDialog};
use flut::{Context, utils::Clock};

// Settings
const SHAKE_DURATION: f32 = 0.5;
const SHAKE_STRENGTH: f32 = 64.0;

pub(crate) struct Shaking {
  accum: f32,
  clock: Clock,
}

impl Shaking {
  #[inline]
  pub(super) const fn new() -> Self {
    let clock = Clock::new(1.0 / consts::UPDATES_PER_SECOND);

    Self { accum: 0.0, clock }
  }

  pub(crate) fn update(mut self, dt: f32, context: &mut Context<'_>) -> State {
    self.accum += dt;

    if !self.clock.update(dt) {
      return State::Shaking(self);
    }

    if self.accum < SHAKE_DURATION {
      context.renderer.set_cam_position(Some((
        fastrand::f32() * SHAKE_STRENGTH,
        fastrand::f32() * SHAKE_STRENGTH,
      )));
      State::Shaking(self)
    } else {
      self.accum = SHAKE_DURATION;
      context.renderer.set_cam_position(None);
      State::ShowingDialog(ShowingDialog::new(context))
    }
  }
}
