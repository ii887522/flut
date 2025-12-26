pub struct Clock {
  interval: f32,
  accum: f32,
}

impl Clock {
  #[inline]
  pub const fn new(interval: f32) -> Self {
    Self {
      interval,
      accum: 0.0,
    }
  }

  pub fn update(&mut self, dt: f32) -> bool {
    self.accum += dt;

    if self.accum < self.interval {
      return false;
    }

    self.accum -= self.interval;
    true
  }
}
