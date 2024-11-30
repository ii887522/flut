use super::Animation;

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct Timer {
  duration: f32,
  accumulator: f32,
  animation: Animation,
}

impl Timer {
  pub fn new(duration: f32) -> Self {
    Self {
      duration,
      accumulator: 0.0,
      animation: Animation::new(),
    }
  }

  pub const fn get_progress(&self) -> f32 {
    self.accumulator / self.duration
  }

  pub fn update(mut self, dt: f32) -> Option<Self> {
    self.accumulator += dt;

    if self.accumulator < self.duration {
      Some(self)
    } else {
      None
    }
  }
}
