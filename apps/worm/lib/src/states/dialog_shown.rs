use crate::consts;
use flut::{
  Context,
  event::Event,
  models::{Align, Icon, RoundRect, Text},
  mouse::MouseButton,
  renderers::renderer_ref,
  utils::sdf,
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
  icon_render_id: renderer_ref::Id,
  title_render_id: renderer_ref::Id,
  desc_render_id: renderer_ref::Id,
  desc: String,
  accum: f32,
  mouse_down_outside_dialog: bool,
  state: State,
}

impl DialogShown {
  pub(super) const fn new(
    dialog_render_id: renderer_ref::Id,
    icon_render_id: renderer_ref::Id,
    title_render_id: renderer_ref::Id,
    desc_render_id: renderer_ref::Id,
    desc: String,
  ) -> Self {
    Self {
      dialog_render_id,
      icon_render_id,
      title_render_id,
      desc_render_id,
      desc,
      accum: 0.0,
      mouse_down_outside_dialog: false,
      state: State::Scaled,
    }
  }

  pub(crate) fn process_event(mut self, event: Event, context: &Context<'_>) -> Self {
    let State::Scaled = self.state else {
      return self;
    };

    match event {
      Event::MouseButtonDown {
        mouse_btn: MouseButton::Left,
        x,
        y,
        ..
      } => {
        // Calculate how far away the mouse is from the dialog
        let dialog_sd = sdf::calc_round_rect_sd(
          (
            x / context.window_content_scale,
            y / context.window_content_scale,
          ),
          RoundRect {
            position: consts::DIALOG_POSITION,
            size: consts::DIALOG_SIZE,
            color: consts::DIALOG_COLOR,
            radius: consts::DIALOG_BORDER_RADIUS,
          },
        );

        self.mouse_down_outside_dialog = dialog_sd > 0.0;
      }
      Event::MouseButtonUp {
        mouse_btn: MouseButton::Left,
        x,
        y,
        ..
      } if self.mouse_down_outside_dialog => {
        // Calculate how far away the mouse is from the dialog
        let dialog_sd = sdf::calc_round_rect_sd(
          (
            x / context.window_content_scale,
            y / context.window_content_scale,
          ),
          RoundRect {
            position: consts::DIALOG_POSITION,
            size: consts::DIALOG_SIZE,
            color: consts::DIALOG_COLOR,
            radius: consts::DIALOG_BORDER_RADIUS,
          },
        );

        // Mouse is outside the dialog
        if dialog_sd > 0.0 {
          // Trigger dialog vibration
          self.state = State::ScalingDown;
        }

        self.mouse_down_outside_dialog = false;
      }
      _ => {}
    }

    self
  }

  fn scale_down(mut self, dt: f32, context: &mut Context<'_>) -> Self {
    self.accum += dt;

    if self.accum > SCALE_DURATION {
      self.accum = SCALE_DURATION;
    }

    let t = self.accum / SCALE_DURATION;

    // Dialog
    let dialog_scale = 1.0 - t * (1.0 - MIN_VIBRATE_SCALE);
    let dialog_size = (
      consts::DIALOG_SIZE.0 * dialog_scale,
      consts::DIALOG_SIZE.1 * dialog_scale,
    );
    let dialog_position = (
      (consts::WINDOW_SIZE.0 as f32 - dialog_size.0) * 0.5,
      (consts::WINDOW_SIZE.1 as f32 - dialog_size.1) * 0.5,
      0,
    );

    // Icon
    let icon_position = (
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.0 as f32 * 0.5
        + dialog_scale * consts::DIALOG_ICON_POSITION.0,
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.1 as f32 * 0.5
        + dialog_scale * consts::DIALOG_ICON_POSITION.1,
      consts::DIALOG_ICON_POSITION.2,
    );
    let icon_size = consts::DIALOG_ICON_SIZE * dialog_scale;

    // Title
    let title_position = (
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.0 as f32 * 0.5
        + dialog_scale * consts::DIALOG_TITLE_POSITION.0,
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.1 as f32 * 0.5
        + dialog_scale * consts::DIALOG_TITLE_POSITION.1,
      consts::DIALOG_TITLE_POSITION.2,
    );
    let title_font_size = consts::DIALOG_TITLE_FONT_SIZE * dialog_scale;

    // Description
    let desc_position = (
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.0 as f32 * 0.5
        + dialog_scale * consts::DIALOG_DESC_POSITION.0,
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.1 as f32 * 0.5
        + dialog_scale * consts::DIALOG_DESC_POSITION.1,
      consts::DIALOG_DESC_POSITION.2,
    );
    let desc_font_size = consts::DIALOG_DESC_FONT_SIZE * dialog_scale;

    context.renderer.update_model(
      self.dialog_render_id,
      RoundRect {
        position: dialog_position,
        size: dialog_size,
        color: consts::DIALOG_COLOR,
        radius: consts::DIALOG_BORDER_RADIUS,
      },
    );

    context.renderer.update_icon(
      self.icon_render_id,
      Icon {
        position: icon_position,
        font_size: icon_size,
        color: consts::DIALOG_ICON_COLOR,
        codepoint: consts::SENTIMENT_VERY_DISSATISFIED,
      },
    );

    context.renderer.update_text(
      self.title_render_id,
      Text {
        position: title_position,
        font_size: title_font_size,
        color: consts::DIALOG_TITLE_COLOR,
        align: Align::Left,
        text: consts::DIALOG_TITLE.into(),
      },
    );

    context.renderer.update_text(
      self.desc_render_id,
      Text {
        position: desc_position,
        font_size: desc_font_size,
        color: consts::DIALOG_DESC_COLOR,
        align: Align::Left,
        text: self.desc.clone().into(),
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

    // Dialog
    let dialog_scale = MIN_VIBRATE_SCALE + t * (1.0 - MIN_VIBRATE_SCALE);
    let dialog_size = (
      consts::DIALOG_SIZE.0 * dialog_scale,
      consts::DIALOG_SIZE.1 * dialog_scale,
    );
    let dialog_position = (
      (consts::WINDOW_SIZE.0 as f32 - dialog_size.0) * 0.5,
      (consts::WINDOW_SIZE.1 as f32 - dialog_size.1) * 0.5,
      0,
    );

    // Icon
    let icon_position = (
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.0 as f32 * 0.5
        + dialog_scale * consts::DIALOG_ICON_POSITION.0,
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.1 as f32 * 0.5
        + dialog_scale * consts::DIALOG_ICON_POSITION.1,
      consts::DIALOG_ICON_POSITION.2,
    );
    let icon_size = consts::DIALOG_ICON_SIZE * dialog_scale;

    // Title
    let title_position = (
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.0 as f32 * 0.5
        + dialog_scale * consts::DIALOG_TITLE_POSITION.0,
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.1 as f32 * 0.5
        + dialog_scale * consts::DIALOG_TITLE_POSITION.1,
      consts::DIALOG_TITLE_POSITION.2,
    );
    let title_font_size = consts::DIALOG_TITLE_FONT_SIZE * dialog_scale;

    // Description
    let desc_position = (
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.0 as f32 * 0.5
        + dialog_scale * consts::DIALOG_DESC_POSITION.0,
      (1.0 - dialog_scale) * consts::WINDOW_SIZE.1 as f32 * 0.5
        + dialog_scale * consts::DIALOG_DESC_POSITION.1,
      consts::DIALOG_DESC_POSITION.2,
    );
    let desc_font_size = consts::DIALOG_DESC_FONT_SIZE * dialog_scale;

    context.renderer.update_model(
      self.dialog_render_id,
      RoundRect {
        position: dialog_position,
        size: dialog_size,
        color: consts::DIALOG_COLOR,
        radius: consts::DIALOG_BORDER_RADIUS,
      },
    );

    context.renderer.update_icon(
      self.icon_render_id,
      Icon {
        position: icon_position,
        font_size: icon_size,
        color: consts::DIALOG_ICON_COLOR,
        codepoint: consts::SENTIMENT_VERY_DISSATISFIED,
      },
    );

    context.renderer.update_text(
      self.title_render_id,
      Text {
        position: title_position,
        font_size: title_font_size,
        color: consts::DIALOG_TITLE_COLOR,
        align: Align::Left,
        text: consts::DIALOG_TITLE.into(),
      },
    );

    context.renderer.update_text(
      self.desc_render_id,
      Text {
        position: desc_position,
        font_size: desc_font_size,
        color: consts::DIALOG_DESC_COLOR,
        align: Align::Left,
        text: self.desc.clone().into(),
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
