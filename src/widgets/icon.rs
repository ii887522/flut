use crate::{
  Transition,
  models::{self, IconName},
};

#[derive(Clone, Copy)]
pub(super) struct Icon {
  pub(super) position: (Transition, Transition, Transition),
  pub(super) size: (Transition, Transition),
  pub(super) color: (u8, u8, u8, u8),
  pub(super) name: IconName,
  pub(super) drawable_id: u16,
}

impl Icon {
  pub(super) fn update(&mut self, dt: f32) -> bool {
    self.position.0.update(dt)
      & self.position.1.update(dt)
      & self.size.0.update(dt)
      & self.size.1.update(dt)
  }
}

impl From<Icon> for models::Icon {
  fn from(icon: Icon) -> Self {
    Self::new(
      (
        icon.position.0.get_value(),
        icon.position.1.get_value(),
        icon.position.2.get_value(),
      ),
      (icon.size.0.get_value(), icon.size.1.get_value()),
      icon.color,
      icon.name,
    )
  }
}
