use crate::{Engine, Transition, models};
use optarg2chain::optarg_impl;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub(super) struct Text {
  position: (f32, f32, f32),
  size: f32,
  max_width: f32,
  color: (u8, u8, u8, u8),
  text: Cow<'static, str>,
  scale: Transition,
  scale_origin: (f32, f32),
  drawable_id: u16,
  prev_scaling: bool,
  scaling: bool,
}

#[optarg_impl]
impl Text {
  #[optarg_method(TextNewBuilder, call)]
  pub(super) fn new(
    #[optarg_default] position: (f32, f32, f32),
    #[optarg(26.0)] size: f32,
    #[optarg(f32::MAX)] max_width: f32,
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    #[optarg_default] text: Cow<'static, str>,
    #[optarg(Transition::new(1.0, 1.0, 0.001))] scale: Transition,
    #[optarg_default] scale_origin: (f32, f32),
  ) -> Self {
    Self {
      position,
      size,
      max_width,
      color,
      text,
      scale,
      scale_origin,
      drawable_id: u16::MAX,
      prev_scaling: true,
      scaling: true,
    }
  }

  pub(super) fn init(&mut self, engine: &mut Engine) {
    self.drawable_id = engine.add_text(models::Text::from(self.clone()));
  }

  pub(super) fn get_position(&self) -> (f32, f32, f32) {
    self.position
  }

  pub(super) fn set_position(&mut self, position: (f32, f32, f32)) {
    self.position = position;
  }

  pub(super) fn set_scale(&mut self, scale: Transition) {
    self.scale = scale;
    self.prev_scaling = true;
    self.scaling = true;
  }

  pub(super) fn update(&mut self, dt: f32, engine: &mut Engine) -> bool {
    self.prev_scaling = self.scaling;
    let done_scaling = self.scale.update(dt);

    if done_scaling {
      self.scaling = false;
    }

    if self.prev_scaling {
      engine.remove_text(self.drawable_id);
    }

    done_scaling
  }

  pub(super) fn finish_update(&mut self, engine: &mut Engine) {
    if self.prev_scaling {
      self.drawable_id = engine.add_text(models::Text::from(self.clone()));
    }
  }
}

impl From<Text> for models::Text {
  fn from(text: Text) -> Self {
    let scale = text.scale.get_value();

    Self::new(
      (
        crate::map(scale, 0.0, 1.0, text.scale_origin.0, text.position.0),
        crate::map(scale, 0.0, 1.0, text.scale_origin.1, text.position.1),
        text.position.2,
      ),
      text.size * scale,
      text.color,
      text.text,
    )
    .max_width(text.max_width * scale)
    .call()
  }
}
