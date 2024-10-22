use crate::{boot::context, models::PulseEvent};
use std::{
  ops::Deref,
  sync::atomic::{AtomicU32, Ordering},
};

static ANIMATION_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnimationCount(u32);

impl AnimationCount {
  pub const fn new() -> Self {
    Self(0)
  }

  pub(crate) fn get() -> u32 {
    ANIMATION_COUNT.load(Ordering::Relaxed)
  }

  pub fn incr(&mut self) {
    self.0 += 1;
    ANIMATION_COUNT.fetch_add(1, Ordering::Relaxed);

    if Self::get() == 1 {
      // Wake up the app event loop
      let event_sender = context::EVENT_SENDER.get().unwrap();
      event_sender.push_custom_event(PulseEvent).unwrap();
    }
  }
}

impl Deref for AnimationCount {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Drop for AnimationCount {
  fn drop(&mut self) {
    ANIMATION_COUNT.fetch_sub(self.0, Ordering::Relaxed);
  }
}
