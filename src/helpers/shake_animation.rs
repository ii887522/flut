use super::{Animation, Clock};
use core::f32;
use optarg2chain::optarg_impl;
use std::f32::consts;

pub trait ShakeAnimationState {
  fn get_translation(&self) -> (f32, f32);
}

impl ShakeAnimationState for Idle {
  fn get_translation(&self) -> (f32, f32) {
    (0.0, 0.0)
  }
}

impl ShakeAnimationState for Move {
  fn get_translation(&self) -> (f32, f32) {
    self.translation
  }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Idle;

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct Move {
  translation: (f32, f32),
  count: u32,
  animation: Animation,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ShakeAnimationAny {
  Idle(ShakeAnimation<Idle>),
  Move(ShakeAnimation<Move>),
}

impl ShakeAnimationAny {
  pub fn get_translation(&self) -> (f32, f32) {
    match self {
      Self::Idle(shake_animation) => shake_animation.get_translation(),
      Self::Move(shake_animation) => shake_animation.get_translation(),
    }
  }

  pub fn as_idle(self) -> Result<ShakeAnimation<Idle>, Self> {
    match self {
      Self::Idle(shake_animation) => Ok(shake_animation),
      _ => Err(self),
    }
  }
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct ShakeAnimation<State: ShakeAnimationState> {
  duration: f32,
  strength: f32,
  clock: Clock,
  state: State,
}

impl<State: ShakeAnimationState> ShakeAnimation<State> {
  pub fn get_translation(&self) -> (f32, f32) {
    self.state.get_translation()
  }
}

#[optarg_impl]
impl ShakeAnimation<Idle> {
  #[optarg_method(ShakeAnimationIdleNewBuilder, call)]
  pub fn new(
    #[optarg(1.0)] duration: f32,
    #[optarg(4.0)] strength: f32,
    #[optarg(30.0)] tps: f32,
  ) -> Self {
    Self {
      duration,
      strength,
      clock: Clock::new(tps),
      state: Idle,
    }
  }

  pub fn shake(self) -> ShakeAnimation<Move> {
    ShakeAnimation {
      duration: self.duration,
      strength: self.strength,
      clock: self.clock,
      state: Move {
        translation: (0.0, 0.0),
        count: 0,
        animation: Animation::new(),
      },
    }
  }
}

impl ShakeAnimation<Move> {
  pub fn update(mut self, dt: f32) -> ShakeAnimationAny {
    if !self.clock.update(dt) {
      return ShakeAnimationAny::Move(self);
    }

    let new_count = self.state.count + 1;
    let max_count = (self.clock.get_tps() * self.duration) as _;

    if new_count >= max_count {
      let new_self = ShakeAnimation {
        duration: self.duration,
        strength: self.strength,
        clock: self.clock,
        state: Idle,
      };

      return ShakeAnimationAny::Idle(new_self);
    }

    let rand_strength = fastrand::f32() * self.strength;
    let rand_angle = fastrand::f32() * consts::TAU;
    let (rand_y, rand_x) = rand_angle.sin_cos();
    let new_translation = (rand_x * rand_strength, rand_y * rand_strength);

    let new_self = ShakeAnimation {
      state: Move {
        translation: new_translation,
        count: new_count,
        ..self.state
      },
      ..self
    };

    ShakeAnimationAny::Move(new_self)
  }
}
