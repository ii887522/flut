use super::{widget::*, BuilderWidget, RectWidget, Scale, Stack, StackChild, Widget};
use crate::{
  boot::context,
  helpers::{transition::MaybeTransition, Transition},
};
use optarg2chain::optarg_impl;
use replace_with::replace_with_or_abort;
use skia_safe::{Color, Rect};
use std::sync::atomic::Ordering;

const SIZE: (f32, f32) = (512.0, 256.0);

pub struct Dialog {
  color: Color,
  background_alpha: MaybeTransition,
  scale: MaybeTransition,
}

#[optarg_impl]
impl Dialog {
  #[optarg_method(DialogNewBuilder, call)]
  pub fn new(#[optarg(Color::BLACK)] color: Color) -> Self {
    Self {
      color,
      background_alpha: MaybeTransition::Move(Transition::new(0.0, 128.0).duration(0.125).call()),
      scale: MaybeTransition::Move(Transition::new(0.0, 1.0).duration(0.125).call()),
    }
  }
}

impl<'a> BuilderWidget<'a> for Dialog {
  fn get_size(&self) -> (f32, f32) {
    // (0.0, 0.0) so that this widget can be inserted in Column or Row or any other layout widget.
    // Size is ignored and this widget always cover the whole app
    (0.0, 0.0)
  }

  fn update(&mut self, dt: f32) -> bool {
    let is_animating = matches!(self.background_alpha, MaybeTransition::Move(_))
      || matches!(self.scale, MaybeTransition::Move(_));

    replace_with_or_abort(
      &mut self.background_alpha,
      |background_alpha| match background_alpha {
        MaybeTransition::Idle(_) => background_alpha,
        MaybeTransition::Move(background_alpha) => background_alpha.update(dt),
      },
    );

    replace_with_or_abort(&mut self.scale, |scale| match scale {
      MaybeTransition::Idle(_) => scale,
      MaybeTransition::Move(scale) => scale.update(dt),
    });

    is_animating
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let drawable_size = (
      context::DRAWABLE_SIZE.0.load(Ordering::Relaxed),
      context::DRAWABLE_SIZE.1.load(Ordering::Relaxed),
    );

    Stack {
      children: vec![
        // Background
        StackChild {
          position: (0.0, 0.0),
          size: drawable_size,
          child: RectWidget {
            color: Color::from_argb(self.background_alpha.get_now() as _, 0, 0, 0),
            ..Default::default()
          }
          .into_widget(),
        },
        // Foreground
        StackChild {
          position: (
            (drawable_size.0 - SIZE.0) * 0.5,
            (drawable_size.1 - SIZE.1) * 0.5,
          ),
          size: SIZE,
          child: Scale {
            scale: self.scale.get_now(),
            child: RectWidget {
              color: self.color,
              border_radius: 8.0,
            }
            .into_widget(),
          }
          .into_widget(),
        },
      ],
    }
    .into()
  }
}
