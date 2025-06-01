use crate::{Engine, Transition, models::Rect};
use optarg2chain::optarg_impl;

#[derive(Clone, Copy, Debug)]
pub(super) struct Glass {
  size: (f32, f32),
  alpha: Transition,
  drawable_id: u16,
  fading: bool,
}

#[optarg_impl]
impl Glass {
  #[optarg_method(GlassNewBuilder, call)]
  pub(super) fn new(
    size: (f32, f32),
    #[optarg(Transition::new(128.0, 128.0, 0.001))] alpha: Transition,
  ) -> Self {
    Self {
      size,
      alpha,
      drawable_id: u16::MAX,
      fading: true,
    }
  }

  pub(super) fn init(&mut self, engine: &mut Engine) {
    self.drawable_id = engine.add_rect(Rect::from(*self));
  }

  pub(super) fn update(&mut self, dt: f32, engine: &mut Engine) -> bool {
    let prev_fading = self.fading;
    let done_fading = self.alpha.update(dt);

    if done_fading {
      self.fading = false;
    }

    if prev_fading {
      engine.update_rect(self.drawable_id, Rect::from(*self));
    }

    done_fading
  }
}

impl From<Glass> for Rect {
  fn from(glass: Glass) -> Self {
    Self::new(
      (0.0, 0.0, 1.0),
      glass.size,
      (0, 0, 0, glass.alpha.get_value() as _),
    )
  }
}
