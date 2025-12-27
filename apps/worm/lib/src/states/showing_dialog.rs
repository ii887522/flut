use crate::{State, consts, states::DialogShown};
use flut::{
  Context,
  models::{Rect, RoundRect},
  renderers::renderer_ref,
};

// Settings
const SHOW_DURATION: f32 = 0.25;
const MAX_GLASS_ALPHA: f32 = 0.5;

pub(crate) struct ShowingDialog {
  glass_render_id: renderer_ref::Id,
  dialog_render_id: renderer_ref::Id,
  accum: f32,
}

impl ShowingDialog {
  pub(super) fn new(context: &mut Context<'_>) -> Self {
    let glass_render_id = context.renderer.add_rect(Rect {
      position: (0.0, 0.0),
      size: (consts::WINDOW_SIZE.0 as _, consts::WINDOW_SIZE.1 as _),
      color: (0.0, 0.0, 0.0, 0.0),
    });

    let dialog_render_id = context.renderer.add_round_rect(RoundRect {
      position: (consts::WINDOW_SIZE.0 as _, consts::WINDOW_SIZE.1 as _),
      size: (0.0, 0.0),
      color: consts::DIALOG_COLOR,
      radius: consts::DIALOG_BORDER_RADIUS,
    });

    Self {
      glass_render_id,
      dialog_render_id,
      accum: 0.0,
    }
  }

  pub(crate) fn update(mut self, dt: f32, context: &mut Context<'_>) -> State {
    self.accum += dt;

    if self.accum > SHOW_DURATION {
      self.accum = SHOW_DURATION;
    }

    let t = self.accum / SHOW_DURATION;
    let dialog_alpha = t * MAX_GLASS_ALPHA;
    let dialog_size = (consts::MAX_DIALOG_SIZE.0 * t, consts::MAX_DIALOG_SIZE.1 * t);
    let dialog_position = (
      (consts::WINDOW_SIZE.0 as f32 - dialog_size.0) * 0.5,
      (consts::WINDOW_SIZE.1 as f32 - dialog_size.1) * 0.5,
    );

    context.renderer.update_rect(
      self.glass_render_id,
      Rect {
        position: (0.0, 0.0),
        size: (consts::WINDOW_SIZE.0 as _, consts::WINDOW_SIZE.1 as _),
        color: (0.0, 0.0, 0.0, dialog_alpha),
      },
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

    if self.accum < SHOW_DURATION {
      State::ShowingDialog(self)
    } else {
      State::DialogShown(DialogShown::new(self.dialog_render_id))
    }
  }
}
