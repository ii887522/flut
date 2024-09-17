use std::ops::{Add, Mul};

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Animation<T> {
  start: T,
  now: T,
  end: T,
  duration: f32,
  old_accumulator: f32,
  new_accumulator: f32,
}

impl<T: Copy> Animation<T> {
  pub const fn new(start: T, end: T, duration: f32) -> Self {
    Self {
      start,
      now: start,
      end,
      duration,
      old_accumulator: 0.0,
      new_accumulator: 0.0,
    }
  }

  pub const fn get_now(&self) -> T {
    self.now
  }
}

impl<T> Animation<T> {
  pub fn is_ended(&self) -> bool {
    self.new_accumulator == self.duration
  }

  pub fn is_just_ended(&self) -> bool {
    self.old_accumulator != self.new_accumulator && self.new_accumulator == self.duration
  }
}

impl<T: Copy + Mul<f32>> Animation<T> {
  pub fn update(&mut self, dt: f32) -> bool
  where
    <T as Mul<f32>>::Output: Add,
    T: From<<<T as Mul<f32>>::Output as Add>::Output>,
  {
    self.old_accumulator = self.new_accumulator;
    self.new_accumulator = (self.new_accumulator + dt).min(self.duration);
    let t = self.new_accumulator / self.duration;
    self.now = (self.start * (1.0 - t) + self.end * t).into();
    self.old_accumulator != self.new_accumulator
  }
}
