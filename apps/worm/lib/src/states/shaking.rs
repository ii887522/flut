use crate::consts;
use flut::{Context, utils::Clock};

// Settings
const SHAKE_DURATION: f32 = 0.5;
const SHAKE_STRENGTH: f32 = 64.0;

pub(crate) struct Shaking {
  shake_accum: f32,
  clock: Clock,
}

impl Shaking {
  #[inline]
  pub(super) const fn new() -> Self {
    let clock = Clock::new(1.0 / consts::UPDATES_PER_SECOND);

    Self {
      shake_accum: 0.0,
      clock,
    }
  }

  pub(crate) fn update(mut self, dt: f32, context: &mut Context<'_>) -> Self {
    self.shake_accum += dt;

    if !self.clock.update(dt) {
      return self;
    }

    if self.shake_accum < SHAKE_DURATION {
      context.renderer.set_cam_position(Some((
        fastrand::f32() * SHAKE_STRENGTH,
        fastrand::f32() * SHAKE_STRENGTH,
      )));
    } else {
      self.shake_accum = SHAKE_DURATION;
      context.renderer.set_cam_position(None);
    }

    self
  }
}
