pub struct Clock {
  interval: f32,
  t: f32,
}

impl Clock {
  pub const fn new(interval: f32) -> Self {
    Self { interval, t: 0.0 }
  }

  pub fn update(&mut self, dt: f32) -> bool {
    self.t += dt;

    if self.t < self.interval {
      return false;
    }

    self.t -= self.interval;
    true
  }
}
