use super::{widget::*, BuilderWidget, Icon, RectWidget, Scale, Stack, StackChild, Text, Widget};
use crate::{
  boot::context,
  helpers::{transition::MaybeTransition, Transition},
  models::{FontCfg, IconName},
};
use optarg2chain::optarg_impl;
use replace_with::replace_with_or_abort;
use sdl2::event::Event;
use skia_safe::{font_style::Weight, Color, Rect};
use std::{borrow::Cow, sync::atomic::Ordering};

const SIZE: (f32, f32) = (512.0, 256.0);

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
  background_alpha: MaybeTransition,
  scale: MaybeTransition,
}

#[optarg_impl]
impl Dialog {
  #[optarg_method(DialogNewBuilder, call)]
  pub fn new(#[optarg(Color::BLACK)] color: Color, #[optarg_default] header: Header) -> Self {
    Self {
      color,
      header,
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

  fn process_event(&mut self, event: &Event) {
    todo!();
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

    let position = (
      (drawable_size.0 - SIZE.0) * 0.5,
      (drawable_size.1 - SIZE.1) * 0.5,
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
          position,
          size: SIZE,
          child: Scale {
            scale: self.scale.get_now(),
            child: Stack {
              children: vec![
                Some(StackChild {
                  position,
                  size: SIZE,
                  child: RectWidget {
                    color: self.color,
                    border_radius: 8.0,
                  }
                  .into_widget(),
                }),
                self.header.icon.map(|header_icon| StackChild {
                  position: (position.0 + 16.0, position.1 + 16.0),
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
                    position: (position.0 + 84.0, position.1 + 32.0),
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
