use super::Timer;
use optarg2chain::optarg_impl;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum MaybeTransition {
  Idle(f32),
  Move(Transition),
}

impl MaybeTransition {
  pub const fn get_now(&self) -> f32 {
    match self {
      MaybeTransition::Idle(now) => *now,
      MaybeTransition::Move(transition) => transition.now,
    }
  }
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct Transition {
  timer: Timer,
  from: f32,
  to: f32,
  now: f32,
}

#[optarg_impl]
impl Transition {
  #[optarg_method(TransitionNewBuilder, call)]
  pub fn new(#[optarg(1.0)] duration: f32, from: f32, to: f32) -> Self {
    Self {
      timer: Timer::new(duration),
      from,
      to,
      now: from,
    }
  }

  pub fn update(self, dt: f32) -> MaybeTransition {
    let Some(timer) = self.timer.update(dt) else {
      return MaybeTransition::Idle(self.to);
    };

    let now = timer.get_progress() * (self.to - self.from) + self.from;
    let new_self = Self { timer, now, ..self };
    MaybeTransition::Move(new_self)
  }
}
