use crate::{Clock, Engine};

#[derive(Debug)]
pub struct Shake {
  clock: Clock,
  strength: f32,
  duration: f32,
  t: f32,
}

impl Shake {
  pub const fn new(strength: f32, duration: f32, tps: f32) -> Self {
    Self {
      clock: Clock::new(tps),
      strength,
      duration,
      t: 0.0,
    }
  }

  pub fn update(mut self, dt: f32, engine: &mut Engine) -> Option<Self> {
    self.t += dt;

    if !self.clock.update(dt) {
      return Some(self);
    }

    engine.set_camera_position((
      (fastrand::f32() - 0.5) * self.strength,
      (fastrand::f32() - 0.5) * self.strength,
    ));

    if self.t < self.duration {
      return Some(self);
    }

    None
  }
}
