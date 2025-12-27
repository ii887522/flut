use crate::consts;
use flut::{
  Context, event::Event, models::RoundRect, mouse::MouseButton, renderers::renderer_ref, utils::sdf,
};

// Settings
const SCALE_DURATION: f32 = 0.05;
const MIN_VIBRATE_SCALE: f32 = 0.9;

enum State {
  Scaled,
  ScalingDown,
  ScalingUp,
}

pub(crate) struct DialogShown {
  dialog_render_id: renderer_ref::Id,
  accum: f32,
  state: State,
}

impl DialogShown {
  pub(super) const fn new(dialog_render_id: renderer_ref::Id) -> Self {
    Self {
      dialog_render_id,
      accum: 0.0,
      state: State::Scaled,
    }
  }

  pub(crate) fn process_event(mut self, event: Event, context: &Context<'_>) -> Self {
    if let Event::MouseButtonUp {
      mouse_btn: MouseButton::Left,
      x,
      y,
      ..
    } = event
      && let State::Scaled = self.state
    {
      let dialog_sd = sdf::calc_round_rect_sd(
        (
          x / context.window_content_scale,
          y / context.window_content_scale,
        ),
        RoundRect {
          position: consts::MIN_DIALOG_POSITION,
          size: consts::MAX_DIALOG_SIZE,
          color: consts::DIALOG_COLOR,
          radius: consts::DIALOG_BORDER_RADIUS,
        },
      );

      if dialog_sd > 0.0 {
        self.state = State::ScalingDown;
      }
    }

    self
  }

  fn scale_down(mut self, dt: f32, context: &mut Context<'_>) -> Self {
    self.accum += dt;

    if self.accum > SCALE_DURATION {
      self.accum = SCALE_DURATION;
    }

    let t = self.accum / SCALE_DURATION;
    let dialog_scale = 1.0 - t * (1.0 - MIN_VIBRATE_SCALE);
    let dialog_size = (
      consts::MAX_DIALOG_SIZE.0 * dialog_scale,
      consts::MAX_DIALOG_SIZE.1 * dialog_scale,
    );
    let dialog_position = (
      (consts::WINDOW_SIZE.0 as f32 - dialog_size.0) * 0.5,
      (consts::WINDOW_SIZE.1 as f32 - dialog_size.1) * 0.5,
    );

    context.renderer.update_round_rect(
      self.dialog_render_id,
      RoundRect {
        position: dialog_position,
        size: dialog_size,
        color: consts::DIALOG_COLOR,
        radius: consts::DIALOG_BORDER_RADIUS,
      },
    );

    if self.accum >= SCALE_DURATION {
      self.accum = 0.0;
      self.state = State::ScalingUp;
    }

    self
  }

  fn scale_up(mut self, dt: f32, context: &mut Context<'_>) -> Self {
    self.accum += dt;

    if self.accum > SCALE_DURATION {
      self.accum = SCALE_DURATION;
    }

    let t = self.accum / SCALE_DURATION;
    let dialog_scale = MIN_VIBRATE_SCALE + t * (1.0 - MIN_VIBRATE_SCALE);
    let dialog_size = (
      consts::MAX_DIALOG_SIZE.0 * dialog_scale,
      consts::MAX_DIALOG_SIZE.1 * dialog_scale,
    );
    let dialog_position = (
      (consts::WINDOW_SIZE.0 as f32 - dialog_size.0) * 0.5,
      (consts::WINDOW_SIZE.1 as f32 - dialog_size.1) * 0.5,
    );

    context.renderer.update_round_rect(
      self.dialog_render_id,
      RoundRect {
        position: dialog_position,
        size: dialog_size,
        color: consts::DIALOG_COLOR,
        radius: consts::DIALOG_BORDER_RADIUS,
      },
    );

    if self.accum >= SCALE_DURATION {
      self.accum = 0.0;
      self.state = State::Scaled;
    }

    self
  }

  pub(crate) fn update(self, dt: f32, context: &mut Context<'_>) -> Self {
    match self.state {
      State::Scaled => self,
      State::ScalingDown => self.scale_down(dt, context),
      State::ScalingUp => self.scale_up(dt, context),
    }
  }
}
