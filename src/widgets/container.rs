use crate::{Engine, Transition, models::RoundRect};
use optarg2chain::optarg_impl;

#[derive(Clone, Copy, Debug)]
pub(super) struct Container {
  position: (f32, f32, f32),
  size: (f32, f32),
  color: (Transition, Transition, Transition, Transition),
  border_radius: f32,
  scale: Transition,
  scale_origin: (f32, f32),
  drawable_id: u16,
  scaling: bool,
}

#[optarg_impl]
impl Container {
  #[optarg_method(ContainerNewBuilder, call)]
  pub(super) fn new(
    #[optarg_default] position: (f32, f32, f32),
    size: (f32, f32),
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    #[optarg(10.0)] border_radius: f32,
    #[optarg(Transition::new(1.0, 1.0, 0.001))] scale: Transition,
    #[optarg_default] scale_origin: (f32, f32),
  ) -> Self {
    Self {
      position,
      size,
      color: (
        Transition::new(color.0 as _, color.0 as _, 0.001),
        Transition::new(color.1 as _, color.1 as _, 0.001),
        Transition::new(color.2 as _, color.2 as _, 0.001),
        Transition::new(color.3 as _, color.3 as _, 0.001),
      ),
      border_radius,
      scale,
      scale_origin,
      drawable_id: u16::MAX,
      scaling: true,
    }
  }

  pub(super) fn init(&mut self, engine: &mut Engine) {
    self.drawable_id = engine.add_round_rect(RoundRect::from(*self));
  }

  pub(super) const fn get_position(&self) -> (f32, f32, f32) {
    self.position
  }

  pub(super) fn set_position(&mut self, position: (f32, f32, f32)) {
    self.position = position;
  }

  pub(super) const fn get_size(&self) -> (f32, f32) {
    self.size
  }

  pub(super) fn set_size(&mut self, size: (f32, f32)) {
    self.size = size;
  }

  pub(super) fn set_scale(&mut self, scale: Transition) {
    self.scale = scale;
    self.scaling = true;
  }

  pub(super) const fn get_color(&self) -> (u8, u8, u8, u8) {
    (
      self.color.0.get_value() as _,
      self.color.1.get_value() as _,
      self.color.2.get_value() as _,
      self.color.3.get_value() as _,
    )
  }

  pub(super) fn set_color(&mut self, color: (Transition, Transition, Transition, Transition)) {
    self.color = color;
    self.scaling = true;
  }

  pub(super) fn update(&mut self, dt: f32, engine: &mut Engine) -> bool {
    let prev_scaling = self.scaling;

    let done_scaling = self.scale.update(dt)
      & self.color.0.update(dt)
      & self.color.1.update(dt)
      & self.color.2.update(dt)
      & self.color.3.update(dt);

    if done_scaling {
      self.scaling = false;
    }

    if prev_scaling {
      engine.update_round_rect(self.drawable_id, RoundRect::from(*self));
    }

    done_scaling
  }
}

impl From<Container> for RoundRect {
  fn from(container: Container) -> Self {
    let scale = container.scale.get_value();

    Self::new(
      (
        crate::map(
          scale,
          0.0,
          1.0,
          container.scale_origin.0,
          container.position.0,
        ),
        crate::map(
          scale,
          0.0,
          1.0,
          container.scale_origin.1,
          container.position.1,
        ),
        container.position.2,
      ),
      (container.size.0 * scale, container.size.1 * scale),
      (
        container.color.0.get_value() as _,
        container.color.1.get_value() as _,
        container.color.2.get_value() as _,
        container.color.3.get_value() as _,
      ),
      container.border_radius * scale,
    )
  }
}
