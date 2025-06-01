use crate::{
  Engine, Transition,
  models::{self, IconName},
};
use optarg2chain::optarg_impl;

#[derive(Clone, Copy, Debug)]
pub(super) struct Icon {
  position: (f32, f32, f32),
  size: (f32, f32),
  color: (u8, u8, u8, u8),
  name: IconName,
  scale: Transition,
  scale_origin: (f32, f32),
  drawable_id: u16,
  scaling: bool,
}

#[optarg_impl]
impl Icon {
  #[optarg_method(IconNewBuilder, call)]
  pub(super) fn new(
    #[optarg_default] position: (f32, f32, f32),
    #[optarg((56.0, 64.0))] size: (f32, f32),
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    name: IconName,
    #[optarg(Transition::new(1.0, 1.0, 0.001))] scale: Transition,
    #[optarg_default] scale_origin: (f32, f32),
  ) -> Self {
    Self {
      position,
      size,
      color,
      name,
      scale,
      scale_origin,
      drawable_id: u16::MAX,
      scaling: true,
    }
  }

  pub(super) fn init(&mut self, engine: &mut Engine) {
    self.drawable_id = engine.add_icon(models::Icon::from(*self));
  }

  pub(super) const fn get_position(&self) -> (f32, f32, f32) {
    self.position
  }

  pub(super) fn set_position(&mut self, position: (f32, f32, f32)) {
    self.position = position;
  }

  pub(super) fn set_scale(&mut self, scale: Transition) {
    self.scale = scale;
    self.scaling = true;
  }

  pub(super) fn update(&mut self, dt: f32, engine: &mut Engine) -> bool {
    let prev_scaling = self.scaling;
    let done_scaling = self.scale.update(dt);

    if done_scaling {
      self.scaling = false;
    }

    if prev_scaling {
      engine.update_icon(self.drawable_id, models::Icon::from(*self));
    }

    done_scaling
  }
}

impl From<Icon> for models::Icon {
  fn from(icon: Icon) -> Self {
    let scale = icon.scale.get_value();

    Self::new(
      (
        crate::map(scale, 0.0, 1.0, icon.scale_origin.0, icon.position.0),
        crate::map(scale, 0.0, 1.0, icon.scale_origin.1, icon.position.1),
        icon.position.2,
      ),
      (icon.size.0 * scale, icon.size.1 * scale),
      icon.color,
      icon.name,
    )
  }
}
