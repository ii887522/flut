use crate::{Transition, models::RoundRect};

#[derive(Clone, Copy)]
pub(super) struct Container {
  pub(super) position: (Transition, Transition),
  pub(super) size: (Transition, Transition),
  pub(super) bg_color: (u8, u8, u8, u8),
  pub(super) border_radius: Transition,
  pub(super) drawable_id: u16,
}

impl Container {
  pub(super) fn update(&mut self, dt: f32) -> bool {
    self.position.0.update(dt)
      & self.position.1.update(dt)
      & self.size.0.update(dt)
      & self.size.1.update(dt)
      & self.border_radius.update(dt)
  }
}

impl From<Container> for RoundRect {
  fn from(container: Container) -> Self {
    Self::new(
      (
        container.position.0.get_value(),
        container.position.1.get_value(),
      ),
      (container.size.0.get_value(), container.size.1.get_value()),
      container.bg_color,
      container.border_radius.get_value(),
    )
  }
}
