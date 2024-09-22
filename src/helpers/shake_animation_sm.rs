use super::{Animation, AnimationCount};
use optarg2chain::optarg_impl;
use rand::{thread_rng, Rng};
use std::f32::consts;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum ShakeAnimationState {
  #[default]
  Start,
  Move {
    count: u32,
  },
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ShakeAnimationSM {
  animation_count: AnimationCount,
  x: Animation<f32>,
  y: Animation<f32>,
  state: ShakeAnimationState,
  magnitude: f32,
  strength: f32,
  max_move_count: u32,
}

impl Default for ShakeAnimationSM {
  fn default() -> Self {
    Self::new().call()
  }
}

#[optarg_impl]
impl ShakeAnimationSM {
  #[optarg_method(ShakeAnimationSMNewBuilder, call)]
  pub fn new(
    #[optarg(4.0)] magnitude: f32,
    #[optarg(10.0)] strength: f32,
    #[optarg(1.0)] duration: f32,
  ) -> Self {
    Self {
      animation_count: AnimationCount::new(),
      x: Animation::new(0.0, 0.0, 0.0),
      y: Animation::new(0.0, 0.0, 0.0),
      state: ShakeAnimationState::Start,
      magnitude,
      strength,
      max_move_count: (duration * strength).round() as _,
    }
  }

  pub const fn get_current_translation(&self) -> (f32, f32) {
    (self.x.get_now(), self.y.get_now())
  }

  pub fn update(&mut self, dt: f32) -> bool {
    let is_dirty = self.x.update(dt) | self.y.update(dt);

    if self.x.is_just_ended() || self.y.is_just_ended() {
      match self.state {
        ShakeAnimationState::Move { count } => {
          if count >= self.max_move_count {
            self.animation_count = AnimationCount::new();
            self.x = Animation::new(0.0, 0.0, 0.0);
            self.y = Animation::new(0.0, 0.0, 0.0);
            self.state = ShakeAnimationState::Start;
          } else {
            self.randomize_translation();
            self.state = ShakeAnimationState::Move { count: count + 1 };
          }
        }
        ShakeAnimationState::Start => panic!("Animating while in Start state which is unexpected"),
      }
    }

    is_dirty
  }

  pub fn shake(&mut self) {
    if self.state != ShakeAnimationState::Start {
      return;
    }

    self.animation_count.incr();
    self.randomize_translation();
    self.state = ShakeAnimationState::Move { count: 1 };
  }

  fn randomize_translation(&mut self) {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..consts::TAU);
    let magnitude = rng.gen_range(0.0..=self.magnitude);
    let move_duration = 1.0 / self.strength;
    self.x = Animation::new(0.0, angle.cos() * magnitude, move_duration);
    self.y = Animation::new(0.0, angle.sin() * magnitude, move_duration);
  }
}
