use crate::{models::round_rect::RoundRect, renderer_ref::RendererRef, utils};
use optarg2chain::optarg_impl;

pub struct Ripple {
  position: (f32, f32, f32),
  end_radius: f32,
  start_color: (u8, u8, u8, u8),
  end_color: (u8, u8, u8, u8),
  duration: f32,
  clipped: bool,
  circle_render_id: u32,
  time: f32,
}

#[optarg_impl]
impl Ripple {
  #[optarg_method(RippleNewBuilder, call)]
  pub fn new(
    #[optarg_default] position: (f32, f32, f32),
    #[optarg(48.0)] end_radius: f32,
    #[optarg((255, 255, 255, 255))] start_color: (u8, u8, u8, u8),
    #[optarg((0, 0, 0, 255))] end_color: (u8, u8, u8, u8),
    #[optarg(1.0)] duration: f32,
    #[optarg_default] clipped: bool,
  ) -> Self {
    Self {
      position,
      end_radius,
      start_color,
      end_color,
      duration,
      clipped,
      circle_render_id: u32::MAX,
      time: 0.0,
    }
  }

  pub fn init(&mut self, renderer: &mut RendererRef<'_>) {
    self.circle_render_id = renderer.add_model(
      RoundRect {
        position: self.position,
        radius: 0.0,
        size: (0.0, 0.0),
        color: utils::pack_color(self.start_color),
      },
      self.clipped,
    );
  }

  #[inline]
  pub const fn translate(&mut self, translation: (f32, f32, f32)) {
    self.position.0 += translation.0;
    self.position.1 += translation.1;
    self.position.2 += translation.2;
  }

  #[inline]
  pub const fn set_end_color(&mut self, end_color: (u8, u8, u8, u8)) {
    self.end_color = end_color;
  }

  pub fn update(&mut self, dt: f32, renderer: &mut RendererRef<'_>) {
    self.time = (self.time + dt).min(self.duration);

    if self.is_ended() {
      return;
    }

    let time_scale = self.time / self.duration;
    let radius = self.end_radius * (time_scale * 4.0).min(1.0);
    let color = utils::lerp_color(self.start_color, self.end_color, time_scale);
    let position = (
      self.position.0 - radius,
      self.position.1 - radius,
      self.position.2,
    );
    let size = (radius * 2.0, radius * 2.0);

    renderer.update_model(
      self.circle_render_id,
      RoundRect {
        position,
        radius,
        size,
        color: utils::pack_color(color),
      },
      self.clipped,
    );
  }

  #[must_use]
  #[inline]
  pub const fn is_ended(&self) -> bool {
    self.time >= self.duration
  }

  pub fn drop(self, renderer: &mut RendererRef<'_>) {
    renderer.remove_model::<RoundRect>(self.circle_render_id, self.clipped);
  }
}
