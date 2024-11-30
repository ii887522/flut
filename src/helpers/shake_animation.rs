use super::{Clock, Timer};
use optarg2chain::optarg_impl;
use std::f32::consts;

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct ShakeAnimation {
  timer: Timer,
  strength: f32,
  clock: Clock,
  translation: (f32, f32),
}

#[optarg_impl]
impl ShakeAnimation {
  #[optarg_method(ShakeAnimationNewBuilder, call)]
  pub fn new(
    #[optarg(1.0)] duration: f32,
    #[optarg(4.0)] strength: f32,
    #[optarg(30.0)] tps: f32,
  ) -> Self {
    Self {
      timer: Timer::new(duration),
      strength,
      clock: Clock::new(tps),
      translation: (0.0, 0.0),
    }
  }

  pub const fn get_translation(&self) -> (f32, f32) {
    self.translation
  }

  pub fn update(mut self, dt: f32) -> Option<Self> {
    let timer = self.timer.update(dt)?;

    if !self.clock.update(dt) {
      return Some(Self { timer, ..self });
    }

    let rand_strength = fastrand::f32() * self.strength;
    let rand_angle = fastrand::f32() * consts::TAU;
    let (rand_y, rand_x) = rand_angle.sin_cos();
    let new_translation = (rand_x * rand_strength, rand_y * rand_strength);

    let new_self = Self {
      timer,
      translation: new_translation,
      ..self
    };

    Some(new_self)
  }
}
