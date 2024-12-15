use super::{widget::*, BuilderWidget, Icon, RectWidget, Scale, Stack, StackChild, Text, Widget};
use crate::{
  boot::context,
  helpers::{transition::MaybeTransition, Transition},
  models::{FontCfg, IconName},
};
use optarg2chain::optarg_impl;
use rayon::iter::Either;
use replace_with::replace_with_or_abort;
use sdl2::mouse::MouseButton;
use skia_safe::{font_style::Weight, Color, Contains, Point, Rect};
use std::{borrow::Cow, sync::atomic::Ordering};

const SIZE: (f32, f32) = (512.0, 256.0);

thread_local! {
  static POSITION: (f32, f32) = (
    (context::DRAWABLE_SIZE.0.load(Ordering::Relaxed) - SIZE.0) * 0.5,
    (context::DRAWABLE_SIZE.1.load(Ordering::Relaxed) - SIZE.1) * 0.5,
  );
}

pub struct Header {
  pub icon: Option<IconName>,
  pub icon_color: Color,
  pub title: Cow<'static, str>,
  pub title_color: Color,
  pub title_font_cfg: FontCfg,
}

impl Default for Header {
  fn default() -> Self {
    Self {
      icon: None,
      icon_color: Color::BLACK,
      title: Cow::Borrowed(""),
      title_color: Color::BLACK,
      title_font_cfg: FontCfg {
        font_size: 32,
        font_weight: Weight::SEMI_BOLD,
        ..Default::default()
      },
    }
  }
}

pub struct Dialog {
  color: Color,
  header: Header,
  animation: DialogAnimationAny,
}

#[optarg_impl]
impl Dialog {
  #[optarg_method(DialogNewBuilder, call)]
  pub fn new(#[optarg(Color::BLACK)] color: Color, #[optarg_default] header: Header) -> Self {
    let animation = DialogAnimationAny::new();

    Self {
      color,
      header,
      animation,
    }
  }
}

impl<'a> BuilderWidget<'a> for Dialog {
  fn get_size(&self) -> (f32, f32) {
    // (0.0, 0.0) so that this widget can be inserted in Column or Row or any other layout widget.
    // Size is ignored and this widget always cover the whole app
    (0.0, 0.0)
  }

  fn on_mouse_up(&mut self, mouse_btn: MouseButton, mouse_position: (f32, f32)) {
    if mouse_btn != MouseButton::Left
      || POSITION
        .with(|&position| Rect::from_xywh(position.0, position.1, SIZE.0, SIZE.1))
        .contains(Point::new(mouse_position.0, mouse_position.1))
    {
      return;
    }

    if let DialogAnimationAny::PoppedUp(popped_up) = self.animation {
      self.animation = DialogAnimationAny::ScalingDown(popped_up.vibrate());
    }
  }

  fn update(&mut self, dt: f32) -> bool {
    let is_animating = matches!(
      self.animation,
      DialogAnimationAny::PoppingUp(_)
        | DialogAnimationAny::ScalingDown(_)
        | DialogAnimationAny::ScalingUp(_)
    );

    replace_with_or_abort(&mut self.animation, |animation| match animation {
      DialogAnimationAny::PoppingUp(popping_up) => match popping_up.update(dt) {
        Either::Left(popping_up) => DialogAnimationAny::PoppingUp(popping_up),
        Either::Right(popped_up) => DialogAnimationAny::PoppedUp(popped_up),
      },
      DialogAnimationAny::PoppedUp(popped_up) => DialogAnimationAny::PoppedUp(popped_up),
      DialogAnimationAny::ScalingDown(scaling_down) => match scaling_down.update(dt) {
        Either::Left(scaling_down) => DialogAnimationAny::ScalingDown(scaling_down),
        Either::Right(scaling_up) => DialogAnimationAny::ScalingUp(scaling_up),
      },
      DialogAnimationAny::ScalingUp(scaling_up) => match scaling_up.update(dt) {
        Either::Left(scaling_up) => DialogAnimationAny::ScalingUp(scaling_up),
        Either::Right(popped_up) => DialogAnimationAny::PoppedUp(popped_up),
      },
    });

    is_animating
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    Stack {
      children: vec![
        // Background
        StackChild {
          position: (0.0, 0.0),
          size: (
            context::DRAWABLE_SIZE.0.load(Ordering::Relaxed),
            context::DRAWABLE_SIZE.1.load(Ordering::Relaxed),
          ),
          child: RectWidget {
            color: Color::from_argb(self.animation.get_background_alpha(), 0, 0, 0),
            ..Default::default()
          }
          .into_widget(),
        },
        // Foreground
        StackChild {
          position: POSITION.with(|&position| position),
          size: SIZE,
          child: Scale {
            scale: self.animation.get_scale(),
            child: Stack {
              children: vec![
                Some(StackChild {
                  position: POSITION.with(|&position| position),
                  size: SIZE,
                  child: RectWidget {
                    color: self.color,
                    border_radius: 8.0,
                  }
                  .into_widget(),
                }),
                self.header.icon.map(|header_icon| StackChild {
                  position: POSITION.with(|&position| (position.0 + 16.0, position.1 + 16.0)),
                  size: (0.0, 0.0),
                  child: Icon::new(header_icon)
                    .size(64)
                    .color(self.header.icon_color)
                    .call()
                    .into_widget(),
                }),
                if self.header.title.is_empty() {
                  None
                } else {
                  Some(StackChild {
                    position: POSITION.with(|&position| (position.0 + 84.0, position.1 + 32.0)),
                    size: (0.0, 0.0),
                    child: Text::new()
                      .text(self.header.title.to_string())
                      .font_cfg(self.header.title_font_cfg)
                      .color(self.header.title_color)
                      .call()
                      .into_widget(),
                  })
                },
              ]
              .into_iter()
              .flatten()
              .collect(),
            }
            .into(),
          }
          .into_widget(),
        },
      ],
    }
    .into()
  }
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct PoppingUp {
  background_alpha: Transition,
  scale: Transition,
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct PoppedUp;

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct ScalingDown {
  scale: Transition,
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct ScalingUp {
  scale: Transition,
}

#[derive(Debug, PartialEq, PartialOrd)]
enum DialogAnimationAny {
  PoppingUp(PoppingUp),
  PoppedUp(PoppedUp),
  ScalingDown(ScalingDown),
  ScalingUp(ScalingUp),
}

impl DialogAnimationAny {
  fn new() -> Self {
    Self::PoppingUp(PoppingUp {
      background_alpha: Transition::new(0.0, 128.0).duration(0.125).call(),
      scale: Transition::new(0.0, 1.0).duration(0.125).call(),
    })
  }

  const fn get_background_alpha(&self) -> u8 {
    match self {
      DialogAnimationAny::PoppingUp(popping_up) => popping_up.background_alpha.get_now() as _,
      DialogAnimationAny::PoppedUp(_)
      | DialogAnimationAny::ScalingDown(_)
      | DialogAnimationAny::ScalingUp(_) => 128,
    }
  }

  const fn get_scale(&self) -> f32 {
    match self {
      DialogAnimationAny::PoppingUp(popping_up) => popping_up.scale.get_now(),
      DialogAnimationAny::PoppedUp(_) => 1.0,
      DialogAnimationAny::ScalingDown(scaling_down) => scaling_down.scale.get_now(),
      DialogAnimationAny::ScalingUp(scaling_up) => scaling_up.scale.get_now(),
    }
  }
}

impl PoppingUp {
  fn update(self, dt: f32) -> Either<Self, PoppedUp> {
    let MaybeTransition::Move(background_alpha) = self.background_alpha.update(dt) else {
      return Either::Right(PoppedUp);
    };

    let MaybeTransition::Move(scale) = self.scale.update(dt) else {
      return Either::Right(PoppedUp);
    };

    Either::Left(Self {
      background_alpha,
      scale,
    })
  }
}

impl PoppedUp {
  fn vibrate(self) -> ScalingDown {
    ScalingDown {
      scale: Transition::new(1.0, 0.9).duration(0.08).call(),
    }
  }
}

impl ScalingDown {
  fn update(self, dt: f32) -> Either<Self, ScalingUp> {
    let MaybeTransition::Move(scale) = self.scale.update(dt) else {
      return Either::Right(ScalingUp {
        scale: Transition::new(0.9, 1.0).duration(0.08).call(),
      });
    };

    Either::Left(Self { scale })
  }
}

impl ScalingUp {
  fn update(self, dt: f32) -> Either<Self, PoppedUp> {
    let MaybeTransition::Move(scale) = self.scale.update(dt) else {
      return Either::Right(PoppedUp);
    };

    Either::Left(Self { scale })
  }
}
