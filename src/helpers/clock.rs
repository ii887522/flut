#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Clock {
  tps: f32,
  accumulator: f32,
}

impl Clock {
  pub const fn new(tps: f32) -> Self {
    Self {
      tps,
      accumulator: 0.0,
    }
  }

  pub fn update(&mut self, dt: f32) -> bool {
    self.accumulator += dt;

    if self.accumulator < 1.0 / self.tps {
      return false;
    }

    self.accumulator -= 1.0 / self.tps;
    true
  }
}
