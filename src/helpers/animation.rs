use std::sync::atomic::{AtomicU32, Ordering};

static ANIMATION_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Animation(());

impl Default for Animation {
  fn default() -> Self {
    Self::new()
  }
}

impl Animation {
  pub fn new() -> Self {
    ANIMATION_COUNT.fetch_add(1, Ordering::Relaxed);
    Self(())
  }

  pub(crate) fn has() -> bool {
    ANIMATION_COUNT.load(Ordering::Relaxed) > 0
  }
}

impl Drop for Animation {
  fn drop(&mut self) {
    ANIMATION_COUNT.fetch_sub(1, Ordering::Relaxed);
  }
}
