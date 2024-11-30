use super::Animation;

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct Clock {
  tps: f32,
  accumulator: f32,
  animation: Animation,
}

impl Clock {
  pub fn new(tps: f32) -> Self {
    Self {
      tps,
      accumulator: 0.0,
      animation: Animation::new(),
    }
  }

  pub const fn get_tps(&self) -> f32 {
    self.tps
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
