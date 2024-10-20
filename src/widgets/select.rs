use super::{
  button::{self, ButtonIcon},
  stateful_widget::State,
  widget::*,
  Button, Column, Icon, ImageWidget, RectWidget, Stack, StackChild, StatefulWidget, Text, Widget,
};
use crate::{
  boot::context,
  helpers::{consts, Animation, AnimationCount},
  models::{icon_name, Lang, Origin, TextStyle},
};
use optarg2chain::optarg_impl;
use rayon::prelude::*;
use sdl2::mouse::MouseButton;
use skia_safe::{
  font_style::{Slant, Weight, Width},
  Color, FontStyle, Rect,
};
use std::{
  borrow::Cow,
  sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelStyle {
  pub font_family: &'static str,
  pub font_style: FontStyle,
  pub font_size: f32,
  pub color: Color,
}

impl Default for LabelStyle {
  fn default() -> Self {
    Self {
      font_family: consts::DEFAULT_FONT_FAMILY,
      font_style: FontStyle::new(Weight::SEMI_BOLD, Width::NORMAL, Slant::Upright),
      font_size: 28.0,
      color: Color::BLACK,
    }
  }
}

impl From<LabelStyle> for TextStyle {
  fn from(style: LabelStyle) -> Self {
    Self {
      font_family: style.font_family,
      font_style: style.font_style,
      font_size: style.font_size,
      color: style.color,
    }
  }
}

impl From<LabelStyle> for button::LabelStyle {
  fn from(style: LabelStyle) -> Self {
    Self {
      font_family: style.font_family,
      font_style: style.font_style,
      font_size: style.font_size,
      color: style.color,
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct SelectOption {
  icon: ButtonIcon,
  label: Cow<'static, str>,
  label_font_family: &'static str,
  bg_color: Color,
}

#[optarg_impl]
impl SelectOption {
  #[optarg_method(SelectOptionNewBuilder, call)]
  pub fn new(
    #[optarg(ButtonIcon::Icon {
    name: 0,
    color: Color::BLACK,
  })]
    icon: ButtonIcon,
    #[optarg_default] label: Cow<'static, str>,
    #[optarg(Lang::En.get_default_font_family())] label_font_family: &'static str,
  ) -> Self {
    Self {
      icon,
      label,
      label_font_family,
      bg_color: Color::WHITE,
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct Select {
  pub is_enabled: bool,
  pub size: (f32, f32),
  pub bg_color: Color,
  pub option_bg_color: Color,
  pub option_hover_bg_color: Color,
  pub border_radius: f32,
  pub is_elevated: bool,
  pub label_style: LabelStyle,
  pub selected_index: u32,
  pub options: Arc<Vec<SelectOption>>,
}

impl Default for Select {
  fn default() -> Self {
    Self {
      is_enabled: true,
      size: (-1.0, -1.0),
      bg_color: Color::WHITE,
      option_bg_color: Color::WHITE,
      option_hover_bg_color: Color::RED,
      border_radius: 8.0,
      is_elevated: true,
      label_style: LabelStyle::default(),
      selected_index: 0,
      options: Arc::new(vec![]),
    }
  }
}

impl<'a> StatefulWidget<'a> for Select {
  fn get_size(&self) -> (f32, f32) {
    self.size
  }

  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(SelectState {
      is_enabled: self.is_enabled,
      bg_color: self.bg_color,
      opt_bg_color: self.option_bg_color,
      border_radius: self.border_radius,
      is_elevated: self.is_elevated,
      label_style: self.label_style,
      selected_index: self.selected_index,
      options: Arc::clone(&self.options),
      is_show_options: false,
      animation_sm: SelectAnimationSM::new(),
    })
  }
}

#[derive(Debug, Default, PartialEq)]
struct SelectState {
  is_enabled: bool,
  bg_color: Color,
  opt_bg_color: Color,
  border_radius: f32,
  is_elevated: bool,
  label_style: LabelStyle,
  selected_index: u32,
  options: Arc<Vec<SelectOption>>,
  is_show_options: bool,
  animation_sm: SelectAnimationSM,
}

impl<'a> State<'a> for SelectState {
  fn on_mouse_over(&mut self, _mouse_position: (f32, f32)) -> bool {
    if !self.is_enabled {
      return true;
    }

    context::HAND_CURSOR.with(|hand_cursor| hand_cursor.set());
    true
  }

  fn on_mouse_out(&mut self, _mouse_position: (f32, f32)) -> bool {
    if !self.is_enabled {
      return true;
    }

    context::ARROW_CURSOR.with(|arrow_cursor| arrow_cursor.set());
    true
  }

  fn on_mouse_up(&mut self, _mouse_position: (f32, f32), _mouse_button: MouseButton) -> bool {
    if !self.is_enabled {
      return true;
    }

    self.is_show_options = !self.is_show_options;

    if self.is_show_options {
      self.animation_sm.fade_in();
    } else {
      self.animation_sm.fade_out();
    }

    true
  }

  fn update(&mut self, dt: f32) -> bool {
    if self.is_enabled {
      return self.animation_sm.update(dt);
    }

    false
  }

  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    let selected_option = &self.options[self.selected_index as usize];
    let option_alpha_ratio = self.animation_sm.alpha.get_now() / 255.0;

    Stack {
      children: vec![
        StackChild {
          position: (constraint.x(), constraint.y()),
          size: (constraint.width(), constraint.height()),
          origin: Origin::TopLeft,
          child: Some(
            RectWidget {
              color: self.bg_color,
              border_radius: self.border_radius,
              is_elevated: self.is_elevated,
            }
            .into_widget(),
          ),
        },
        StackChild {
          position: (
            constraint.x() + 16.0,
            constraint.y() + constraint.height() * 0.5,
          ),
          size: (0.0, 0.0),
          origin: Origin::Left,
          child: Some(match selected_option.icon {
            ButtonIcon::Icon {
              name: icon_name,
              color: icon_color,
            } => Icon::new(icon_name)
              .size(48.0)
              .color(icon_color)
              .call()
              .into_widget(),
            ButtonIcon::Image { file_path, tint } => ImageWidget::new(file_path)
              .size((48.0, 48.0))
              .tint(tint)
              .call()
              .into_widget(),
          }),
        },
        StackChild {
          position: (
            constraint.x() + 80.0,
            constraint.y() + constraint.height() * 0.5,
          ),
          size: (0.0, 0.0),
          origin: Origin::Left,
          child: Some(
            Text::new()
              .text(selected_option.label.to_string())
              .style(TextStyle {
                font_family: selected_option.label_font_family,
                ..self.label_style.into()
              })
              .call()
              .into_widget(),
          ),
        },
        StackChild {
          position: (
            constraint.x() + constraint.width() - 16.0,
            constraint.y() + constraint.height() * 0.5,
          ),
          size: (0.0, 0.0),
          origin: Origin::Right,
          child: Some(
            Icon::new(icon_name::ARROW_DROP_DOWN)
              .size(48.0)
              .color(self.label_style.color)
              .degrees(self.animation_sm.arrow_degrees.get_now())
              .call()
              .into_widget(),
          ),
        },
        StackChild {
          position: (constraint.x(), constraint.y() + constraint.height() + 4.0),
          size: (0.0, 0.0),
          origin: Origin::TopLeft,
          child: if self.is_show_options || self.animation_sm.state == SelectAnimationState::FadeOut
          {
            Some(
              Column::new()
                .children(
                  self
                    .options
                    .par_iter()
                    .map(|option| {
                      Button {
                        bg_color: self
                          .opt_bg_color
                          .with_a((self.opt_bg_color.a() as f32 * option_alpha_ratio) as _),
                        border_radius: 0.0,
                        is_elevated: false,
                        icon: match option.icon {
                          ButtonIcon::Icon { name, color } => ButtonIcon::Icon {
                            name,
                            color: color.with_a((color.a() as f32 * option_alpha_ratio) as _),
                          },
                          ButtonIcon::Image { file_path, tint } => ButtonIcon::Image {
                            file_path,
                            tint: tint.with_a((tint.a() as f32 * option_alpha_ratio) as _),
                          },
                        },
                        label: Cow::Owned(option.label.to_string()),
                        label_style: button::LabelStyle {
                          font_family: option.label_font_family,
                          color: self
                            .label_style
                            .color
                            .with_a((self.label_style.color.a() as f32 * option_alpha_ratio) as _),
                          ..self.label_style.into()
                        },
                        size: (constraint.width(), constraint.height()),
                        on_mouse_over: Arc::new(Mutex::new(|| {
                          //
                        })),
                        on_mouse_out: Arc::new(Mutex::new(|| {
                          //
                        })),
                        on_mouse_up: Arc::new(Mutex::new(|| {})),
                        ..Default::default()
                      }
                      .into_widget()
                    })
                    .collect::<Vec<_>>(),
                )
                .call()
                .into_widget(),
            )
          } else {
            None
          },
        },
      ],
    }
    .into()
  }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum SelectAnimationState {
  #[default]
  Start,
  FadeIn,
  Wait,
  FadeOut,
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct SelectAnimationSM {
  animation_count: AnimationCount,
  arrow_degrees: Animation<f32>,
  alpha: Animation<f32>,
  state: SelectAnimationState,
}

impl SelectAnimationSM {
  fn new() -> Self {
    Self {
      animation_count: AnimationCount::new(),
      arrow_degrees: Animation::new(0.0, 0.0, 0.0),
      alpha: Animation::new(0.0, 0.0, 0.0),
      state: SelectAnimationState::Start,
    }
  }

  fn update(&mut self, dt: f32) -> bool {
    let is_dirty = self.arrow_degrees.update(dt) | self.alpha.update(dt);

    if self.arrow_degrees.is_just_ended() || self.alpha.is_just_ended() {
      match self.state {
        SelectAnimationState::FadeIn => {
          self.animation_count = AnimationCount::new();
          self.state = SelectAnimationState::Wait;
        }
        SelectAnimationState::FadeOut => {
          self.animation_count = AnimationCount::new();
          self.state = SelectAnimationState::Start;
        }
        state @ (SelectAnimationState::Start | SelectAnimationState::Wait) => {
          unreachable!("Animating while in {state:?} state which is unexpected");
        }
      }
    }

    is_dirty
  }

  fn fade_in(&mut self) {
    if !matches!(
      self.state,
      SelectAnimationState::Start | SelectAnimationState::FadeOut
    ) {
      return;
    }

    self.animation_count.incr();
    self.arrow_degrees = Animation::new(self.arrow_degrees.get_now(), 180.0, 0.125);
    self.alpha = Animation::new(self.alpha.get_now(), 255.0, 0.125);
    self.state = SelectAnimationState::FadeIn;
  }

  fn fade_out(&mut self) {
    if !matches!(
      self.state,
      SelectAnimationState::FadeIn | SelectAnimationState::Wait
    ) {
      return;
    }

    self.animation_count.incr();
    self.arrow_degrees = Animation::new(self.arrow_degrees.get_now(), 0.0, 0.125);
    self.alpha = Animation::new(self.alpha.get_now(), 0.0, 0.125);
    self.state = SelectAnimationState::FadeOut;
  }
}
