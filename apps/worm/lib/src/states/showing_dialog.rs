use crate::{State, consts, states::DialogShown};
use flut::{
  Context,
  models::{Align, Icon, Rect, RoundRect, Text},
  renderers::renderer_ref,
};

// General Settings
const SHOW_DURATION: f32 = 0.25;

// Glass settings
const MAX_GLASS_ALPHA: u8 = 128;
const GLASS_POSITION: (f32, f32, u8) = (0.0, 0.0, 0);
const GLASS_COLOR: (u8, u8, u8, u8) = (0, 0, 0, 0);

pub(crate) struct ShowingDialog {
  glass_render_id: renderer_ref::Id,
  dialog_render_id: renderer_ref::Id,
  icon_render_id: renderer_ref::Id,
  title_render_id: renderer_ref::Id,
  desc_render_id: renderer_ref::Id,
  desc: String,
  accum: f32,
}

impl ShowingDialog {
  pub(super) fn new(context: &mut Context<'_>, score: usize) -> Self {
    let glass_render_id = context.renderer.add_model(Rect {
      position: GLASS_POSITION,
      size: (consts::WINDOW_SIZE.0 as _, consts::WINDOW_SIZE.1 as _),
      color: GLASS_COLOR,
    });

    let dialog_render_id = context.renderer.add_model(RoundRect {
      position: (
        consts::WINDOW_SIZE.0 as f32 * 0.5,
        consts::WINDOW_SIZE.1 as f32 * 0.5,
        0,
      ),
      size: (0.0, 0.0),
      color: consts::DIALOG_COLOR,
      radius: consts::DIALOG_BORDER_RADIUS,
    });

    let icon_render_id = context.renderer.add_icon(Icon {
      position: (
        consts::WINDOW_SIZE.0 as f32 * 0.5,
        consts::WINDOW_SIZE.1 as f32 * 0.5,
        1,
      ),
      font_size: 0.0,
      color: consts::DIALOG_ICON_COLOR,
      codepoint: consts::SENTIMENT_VERY_DISSATISFIED,
    });

    let title_render_id = context.renderer.add_text(Text {
      position: (
        consts::WINDOW_SIZE.0 as f32 * 0.5,
        consts::WINDOW_SIZE.1 as f32 * 0.5,
        1,
      ),
      font_size: 0.0,
      color: consts::DIALOG_TITLE_COLOR,
      align: Align::Left,
      text: consts::DIALOG_TITLE.into(),
    });

    let desc = format!(
      "You ate {score} green {apple}! Want to try again?",
      apple = if score != 1 { "apples" } else { "apple" }
    );

    let desc_render_id = context.renderer.add_text(Text {
      position: (
        consts::WINDOW_SIZE.0 as f32 * 0.5,
        consts::WINDOW_SIZE.1 as f32 * 0.5,
        1,
      ),
      font_size: 0.0,
      color: consts::DIALOG_DESC_COLOR,
      align: Align::Left,
      text: desc.clone().into(),
    });

    Self {
      glass_render_id,
      dialog_render_id,
      icon_render_id,
      title_render_id,
      desc_render_id,
      desc,
      accum: 0.0,
    }
  }

  pub(crate) fn update(mut self, dt: f32, context: &mut Context<'_>) -> State {
    self.accum += dt;

    if self.accum > SHOW_DURATION {
      self.accum = SHOW_DURATION;
    }

    let t = self.accum / SHOW_DURATION;

    // Dialog
    let dialog_alpha = (t * MAX_GLASS_ALPHA as f32) as _;
    let dialog_size = (consts::DIALOG_SIZE.0 * t, consts::DIALOG_SIZE.1 * t);
    let dialog_position = (
      (consts::WINDOW_SIZE.0 as f32 - dialog_size.0) * 0.5,
      (consts::WINDOW_SIZE.1 as f32 - dialog_size.1) * 0.5,
      0,
    );

    // Icon
    let icon_position = (
      (1.0 - t) * consts::WINDOW_SIZE.0 as f32 * 0.5 + t * consts::DIALOG_ICON_POSITION.0,
      (1.0 - t) * consts::WINDOW_SIZE.1 as f32 * 0.5 + t * consts::DIALOG_ICON_POSITION.1,
      consts::DIALOG_ICON_POSITION.2,
    );
    let icon_size = consts::DIALOG_ICON_SIZE * t;

    // Title
    let title_position = (
      (1.0 - t) * consts::WINDOW_SIZE.0 as f32 * 0.5 + t * consts::DIALOG_TITLE_POSITION.0,
      (1.0 - t) * consts::WINDOW_SIZE.1 as f32 * 0.5 + t * consts::DIALOG_TITLE_POSITION.1,
      consts::DIALOG_TITLE_POSITION.2,
    );
    let title_font_size = consts::DIALOG_TITLE_FONT_SIZE * t;

    // Description
    let desc_position = (
      (1.0 - t) * consts::WINDOW_SIZE.0 as f32 * 0.5 + t * consts::DIALOG_DESC_POSITION.0,
      (1.0 - t) * consts::WINDOW_SIZE.1 as f32 * 0.5 + t * consts::DIALOG_DESC_POSITION.1,
      consts::DIALOG_DESC_POSITION.2,
    );
    let desc_font_size = consts::DIALOG_DESC_FONT_SIZE * t;

    context.renderer.update_model(
      self.glass_render_id,
      Rect {
        position: GLASS_POSITION,
        size: (consts::WINDOW_SIZE.0 as _, consts::WINDOW_SIZE.1 as _),
        color: (GLASS_COLOR.0, GLASS_COLOR.1, GLASS_COLOR.2, dialog_alpha),
      },
    );

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

    if self.accum < SHOW_DURATION {
      State::ShowingDialog(self)
    } else {
      State::DialogShown(DialogShown::new(
        self.dialog_render_id,
        self.icon_render_id,
        self.title_render_id,
        self.desc_render_id,
        self.desc,
      ))
    }
  }
}
