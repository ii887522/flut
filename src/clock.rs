pub struct Clock {
  tps: f32,
  t: f32,
}

impl Clock {
  pub const fn new(tps: f32) -> Self {
    Self { tps, t: 0.0 }
  }

  pub fn update(&mut self, dt: f32) -> bool {
    self.t += dt;

    if self.t < 1.0 / self.tps {
      return false;
    }

    self.t -= 1.0 / self.tps;
    true
  }
}
