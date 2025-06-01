#[derive(Clone, Copy, Debug)]
pub struct Transition {
  from: f32,
  to: f32,
  duration: f32,
  t: f32,
}

impl Transition {
  pub const fn new(from: f32, to: f32, duration: f32) -> Self {
    Self {
      from,
      to,
      t: 0.0,
      duration,
    }
  }

  pub const fn get_value(&self) -> f32 {
    self.from + (self.to - self.from) * (self.t / self.duration)
  }

  pub fn update(&mut self, dt: f32) -> bool {
    self.t = self.duration.min(self.t + dt);
    self.t >= self.duration
  }
}
