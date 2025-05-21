use crate::{Transition, models};
use std::borrow::Cow;

#[derive(Clone)]
pub(super) struct Text {
  pub(super) position: (Transition, Transition, Transition),
  pub(super) size: Transition,
  pub(super) max_width: Transition,
  pub(super) color: (u8, u8, u8, u8),
  pub(super) text: Cow<'static, str>,
  pub(super) drawable_id: u16,
}

impl Text {
  pub(super) fn update(&mut self, dt: f32) -> bool {
    self.position.0.update(dt)
      & self.position.1.update(dt)
      & self.position.2.update(dt)
      & self.size.update(dt)
      & self.max_width.update(dt)
  }
}

impl From<Text> for models::Text {
  fn from(text: Text) -> Self {
    Self::new(
      (
        text.position.0.get_value(),
        text.position.1.get_value(),
        text.position.2.get_value(),
      ),
      text.size.get_value(),
      text.color,
      text.text,
    )
    .max_width(text.max_width.get_value())
    .call()
  }
}
