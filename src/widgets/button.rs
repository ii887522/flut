use super::{
  stateful_widget::State, widget::*, Icon, RectWidget, Row, Spacing, Stack, StackChild,
  StatefulWidget, Text, Widget,
};
use crate::{
  boot::context,
  helpers::{consts, Animation, AnimationCount},
  models::{Origin, TextStyle, VerticalAlign},
};
use sdl2::mouse::MouseButton;
use skia_safe::{
  font_style::{Slant, Weight, Width},
  BlurStyle, Canvas, ClipOp, Color, FontStyle, MaskFilter, Paint, RRect, Rect,
};
use std::{
  borrow::Cow,
  fmt::{self, Debug, Formatter},
  sync::{atomic::Ordering, Arc, Mutex},
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
    TextStyle {
      font_family: style.font_family,
      font_style: style.font_style,
      font_size: style.font_size,
      color: style.color,
    }
  }
}

pub struct Button<'a> {
  pub is_enabled: bool,
  pub bg_color: Color,
  pub border_radius: f32,
  pub is_elevated: bool,
  pub icon: u16,
  pub icon_color: Color,
  pub label: Cow<'static, str>,
  pub label_style: LabelStyle,
  pub size: (f32, f32),
  pub child_align: VerticalAlign,
  pub is_cursor_fixed: bool,
  pub has_effect: bool,
  pub on_mouse_over: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_mouse_out: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_mouse_down: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_mouse_up: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_right_mouse_down: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_right_mouse_up: Arc<Mutex<dyn FnMut() + 'a + Send>>,
}

impl Debug for Button<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("Button")
      .field("is_enabled", &self.is_enabled)
      .field("bg_color", &self.bg_color)
      .field("border_radius", &self.border_radius)
      .field("is_elevated", &self.is_elevated)
      .field("icon", &self.icon)
      .field("icon_color", &self.icon_color)
      .field("label", &self.label)
      .field("label_style", &self.label_style)
      .field("size", &self.size)
      .field("child_align", &self.child_align)
      .field("is_cursor_fixed", &self.is_cursor_fixed)
      .field("has_effect", &self.has_effect)
      .finish_non_exhaustive()
  }
}

impl Default for Button<'_> {
  fn default() -> Self {
    Self {
      is_enabled: true,
      bg_color: Color::WHITE,
      border_radius: 8.0,
      is_elevated: true,
      icon: 0,
      icon_color: Color::BLACK,
      label: Cow::Borrowed(""),
      label_style: LabelStyle::default(),
      size: (-1.0, -1.0),
      child_align: VerticalAlign::Middle,
      is_cursor_fixed: false,
      has_effect: true,
      on_mouse_over: Arc::new(Mutex::new(|| {})),
      on_mouse_out: Arc::new(Mutex::new(|| {})),
      on_mouse_down: Arc::new(Mutex::new(|| {})),
      on_mouse_up: Arc::new(Mutex::new(|| {})),
      on_right_mouse_down: Arc::new(Mutex::new(|| {})),
      on_right_mouse_up: Arc::new(Mutex::new(|| {})),
    }
  }
}

impl<'a> StatefulWidget<'a> for Button<'a> {
  fn get_size(&self) -> (f32, f32) {
    self.size
  }

  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(ButtonState {
      is_enabled: self.is_enabled,
      bg_color: self.bg_color,
      border_radius: self.border_radius,
      req_is_elevated: self.is_elevated,
      is_elevated: self.is_elevated,
      icon: self.icon,
      icon_color: self.icon_color,
      label: self.label.to_string(),
      label_style: self.label_style,
      child_align: self.child_align,
      is_cursor_fixed: self.is_cursor_fixed,
      has_effect: self.has_effect,
      on_mouse_over: Arc::clone(&self.on_mouse_over),
      on_mouse_out: Arc::clone(&self.on_mouse_out),
      on_mouse_down: Arc::clone(&self.on_mouse_down),
      on_mouse_up: Arc::clone(&self.on_mouse_up),
      on_right_mouse_down: Arc::clone(&self.on_right_mouse_down),
      on_right_mouse_up: Arc::clone(&self.on_right_mouse_up),
      animation_sm: ButtonAnimationSM::new(),
      mouse_down_position: (-1.0, -1.0),
    })
  }
}

struct ButtonState<'a> {
  is_enabled: bool,
  bg_color: Color,
  border_radius: f32,
  req_is_elevated: bool,
  is_elevated: bool,
  icon: u16,
  icon_color: Color,
  label: String,
  label_style: LabelStyle,
  child_align: VerticalAlign,
  is_cursor_fixed: bool,
  has_effect: bool,
  on_mouse_over: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_mouse_out: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_mouse_down: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_mouse_up: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_right_mouse_down: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_right_mouse_up: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  animation_sm: ButtonAnimationSM,
  mouse_down_position: (f32, f32),
}

impl Debug for ButtonState<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("ButtonState")
      .field("is_enabled", &self.is_enabled)
      .field("bg_color", &self.bg_color)
      .field("border_radius", &self.border_radius)
      .field("req_is_elevated", &self.req_is_elevated)
      .field("is_elevated", &self.is_elevated)
      .field("icon", &self.icon)
      .field("icon_color", &self.icon_color)
      .field("label", &self.label)
      .field("label_style", &self.label_style)
      .field("child_align", &self.child_align)
      .field("is_cursor_fixed", &self.is_cursor_fixed)
      .field("has_effect", &self.has_effect)
      .field("animation_sm", &self.animation_sm)
      .field("mouse_down_position", &self.mouse_down_position)
      .finish_non_exhaustive()
  }
}

impl<'a> State<'a> for ButtonState<'_> {
  fn on_mouse_over(&mut self, _mouse_position: (f32, f32)) -> bool {
    if !self.is_enabled {
      return true;
    }

    if !self.is_cursor_fixed {
      context::HAND_CURSOR.with(|hand_cursor| hand_cursor.set());
    }

    self.on_mouse_over.lock().unwrap()();
    true
  }

  fn on_mouse_out(&mut self, _mouse_position: (f32, f32)) -> bool {
    if !self.is_enabled {
      return true;
    }

    if !self.is_cursor_fixed {
      context::ARROW_CURSOR.with(|arrow_cursor| arrow_cursor.set());
    }

    if self.has_effect {
      self.animation_sm.fade_out();
    }

    self.is_elevated = self.req_is_elevated;
    self.on_mouse_out.lock().unwrap()();
    true
  }

  fn on_mouse_down(&mut self, mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if !self.is_enabled {
      return true;
    }

    self.mouse_down_position = mouse_position;

    match mouse_button {
      MouseButton::Left => {
        if self.has_effect {
          self.animation_sm.ripple();
        }

        self.is_elevated = false;
        self.on_mouse_down.lock().unwrap()();
      }
      MouseButton::Right => self.on_right_mouse_down.lock().unwrap()(),
      _ => {}
    }

    true
  }

  fn on_mouse_up(&mut self, _mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if !self.is_enabled {
      return true;
    }

    match mouse_button {
      MouseButton::Left => {
        if self.has_effect {
          self.animation_sm.fade_out();
        }

        self.is_elevated = self.req_is_elevated;
        self.on_mouse_up.lock().unwrap()();
      }
      MouseButton::Right => self.on_right_mouse_up.lock().unwrap()(),
      _ => {}
    }

    true
  }

  fn update(&mut self, dt: f32) -> bool {
    if self.is_enabled && self.has_effect {
      return self.animation_sm.update(dt);
    }

    false
  }

  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    self.animation_sm.constraint = constraint;

    Stack {
      children: vec![
        Some(StackChild {
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
        }),
        if self.icon == 0 && self.label.is_empty() {
          None
        } else {
          Some(StackChild {
            position: (
              constraint.x() + constraint.width() * 0.5,
              constraint.y() + constraint.height() * 0.5,
            ),
            size: (constraint.width(), constraint.height()),
            origin: Origin::Center,
            child: Some(
              Row::new()
                .align(self.child_align)
                .children(
                  vec![
                    if self.icon == 0 {
                      None
                    } else {
                      Some(
                        Icon::new(self.icon)
                          .size(40.0)
                          .color(self.icon_color)
                          .call()
                          .into_widget(),
                      )
                    },
                    if self.icon == 0 || self.label.is_empty() {
                      None
                    } else {
                      Some(
                        Spacing {
                          width: 16.0,
                          ..Default::default()
                        }
                        .into_widget(),
                      )
                    },
                    if self.label.is_empty() {
                      None
                    } else {
                      Some(
                        Text::new()
                          .text(self.label.to_string())
                          .style(self.label_style)
                          .call()
                          .into_widget(),
                      )
                    },
                  ]
                  .into_iter()
                  .flatten()
                  .collect::<Vec<_>>(),
                )
                .call()
                .into_widget(),
            ),
          })
        },
      ]
      .into_iter()
      .flatten()
      .collect::<Vec<_>>(),
    }
    .into()
  }

  fn post_draw(&self, canvas: &Canvas, constraint: Rect) {
    if !self.has_effect
      || self.animation_sm.ripple_radius.get_now() <= 0.0
      || self.animation_sm.ripple_alpha.get_now() <= 0.0
    {
      return;
    }

    canvas.save();

    // Ensure ripple won't draw outside button
    canvas.clip_rrect(
      RRect::new_rect_xy(constraint, self.border_radius, self.border_radius),
      ClipOp::Intersect,
      true,
    );

    // Draw ripple
    canvas.draw_circle(
      self.mouse_down_position,
      self.animation_sm.ripple_radius.get_now(),
      Paint::default()
        .set_anti_alias(true)
        .set_color(Color::from_argb(
          self.animation_sm.ripple_alpha.get_now() as _,
          255,
          255,
          255,
        ))
        .set_mask_filter(MaskFilter::blur(BlurStyle::Normal, 2.0, false)),
    );

    canvas.restore();
  }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum ButtonAnimationState {
  #[default]
  Start,
  Ripple,
  Wait,
  FadeOut,
}

#[derive(Debug, Default, PartialEq)]
struct ButtonAnimationSM {
  animation_count: AnimationCount,
  ripple_radius: Animation<f32>,
  ripple_alpha: Animation<f32>,
  state: ButtonAnimationState,
  constraint: Rect,
}

impl ButtonAnimationSM {
  fn new() -> Self {
    Self {
      animation_count: AnimationCount::new(),
      ripple_radius: Animation::new(0.0, 0.0, 0.0),
      ripple_alpha: Animation::new(0.0, 0.0, 0.0),
      state: ButtonAnimationState::Start,

      // Constraint defaults to app size first, later will be overridden with actual button size in ButtonState::build()
      constraint: Rect::from_size((
        context::DRAWABLE_SIZE.0.load(Ordering::Relaxed),
        context::DRAWABLE_SIZE.1.load(Ordering::Relaxed),
      )),
    }
  }

  fn update(&mut self, dt: f32) -> bool {
    let is_dirty = self.ripple_radius.update(dt) | self.ripple_alpha.update(dt);

    if self.ripple_radius.is_just_ended() || self.ripple_alpha.is_just_ended() {
      match self.state {
        ButtonAnimationState::Ripple => {
          self.animation_count = AnimationCount::new();
          self.state = ButtonAnimationState::Wait;
        }
        ButtonAnimationState::FadeOut => {
          self.animation_count = AnimationCount::new();
          self.state = ButtonAnimationState::Start;
        }
        state @ (ButtonAnimationState::Start | ButtonAnimationState::Wait) => {
          unreachable!("Animating while in {state:?} state which is unexpected");
        }
      }
    }

    is_dirty
  }

  fn ripple(&mut self) {
    if !matches!(
      self.state,
      ButtonAnimationState::Start | ButtonAnimationState::FadeOut
    ) {
      return;
    }

    self.animation_count.incr();

    self.ripple_radius = Animation::new(
      0.0,
      self.constraint.width().max(self.constraint.height()),
      0.25,
    );

    self.ripple_alpha = Animation::new(64.0, 64.0, 0.0);
    self.state = ButtonAnimationState::Ripple;
  }

  fn fade_out(&mut self) {
    if !matches!(
      self.state,
      ButtonAnimationState::Ripple | ButtonAnimationState::Wait
    ) {
      return;
    }

    self.animation_count.incr();

    self.ripple_radius = Animation::new(
      self.ripple_radius.get_now(),
      self.ripple_radius.get_now(),
      0.0,
    );

    self.ripple_alpha = Animation::new(64.0, 0.0, 0.5);
    self.state = ButtonAnimationState::FadeOut;
  }
}
