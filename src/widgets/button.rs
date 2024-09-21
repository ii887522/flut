use super::{
  stateful_widget::State, widget::*, Icon, RectWidget, Row, Spacing, Stack, StackChild,
  StatefulWidget, Text, Widget,
};
use crate::{
  boot::context,
  helpers::{Animation, AnimationCount},
  models::{Origin, VerticalAlign},
};
use sdl2::mouse::MouseButton;
use skia_safe::{
  font_style::{Slant, Weight, Width},
  BlurStyle, Canvas, ClipOp, Color, FontStyle, MaskFilter, Paint, RRect, Rect,
};
use std::{
  fmt::{self, Debug, Formatter},
  sync::{atomic::Ordering, Arc, Mutex},
};

pub struct Button<'a> {
  pub bg_color: Color,
  pub border_radius: f32,
  pub is_elevated: bool,
  pub icon: u16,
  pub icon_color: Color,
  pub label: String,
  pub label_color: Color,
  pub size: (f32, f32),
  pub child_align: VerticalAlign,
  pub on_mouse_over: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  pub on_mouse_out: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  pub on_mouse_down: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  pub on_mouse_up: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
}

impl Debug for Button<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("Button")
      .field("bg_color", &self.bg_color)
      .field("border_radius", &self.border_radius)
      .field("is_elevated", &self.is_elevated)
      .field("icon", &self.icon)
      .field("icon_color", &self.icon_color)
      .field("label", &self.label)
      .field("label_color", &self.label_color)
      .field("size", &self.size)
      .field("child_align", &self.child_align)
      .finish_non_exhaustive()
  }
}

impl Default for Button<'_> {
  fn default() -> Self {
    Self {
      bg_color: Color::WHITE,
      border_radius: 8.0,
      is_elevated: true,
      icon: 0,
      icon_color: Color::BLACK,
      label: "".to_string(),
      label_color: Color::BLACK,
      size: (-1.0, -1.0),
      child_align: VerticalAlign::Middle,
      on_mouse_over: None,
      on_mouse_out: None,
      on_mouse_down: None,
      on_mouse_up: None,
    }
  }
}

impl<'a> StatefulWidget<'a> for Button<'a> {
  fn get_size(&self) -> (f32, f32) {
    self.size
  }

  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(ButtonState {
      bg_color: self.bg_color,
      border_radius: self.border_radius,
      req_is_elevated: self.is_elevated,
      is_elevated: self.is_elevated,
      icon: self.icon,
      icon_color: self.icon_color,
      label: self.label.to_string(),
      label_color: self.label_color,
      child_align: self.child_align,
      on_mouse_over: self.on_mouse_over.take(),
      on_mouse_out: self.on_mouse_out.take(),
      on_mouse_down: self.on_mouse_down.take(),
      on_mouse_up: self.on_mouse_up.take(),
      animation_sm: ButtonAnimationSM::new(),
      mouse_down_position: (-1.0, -1.0),
    })
  }
}

#[derive(Default)]
struct ButtonState<'a> {
  bg_color: Color,
  border_radius: f32,
  req_is_elevated: bool,
  is_elevated: bool,
  icon: u16,
  icon_color: Color,
  label: String,
  label_color: Color,
  child_align: VerticalAlign,
  on_mouse_over: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  on_mouse_out: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  on_mouse_down: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  on_mouse_up: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  animation_sm: ButtonAnimationSM,
  mouse_down_position: (f32, f32),
}

impl Debug for ButtonState<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("ButtonState")
      .field("bg_color", &self.bg_color)
      .field("border_radius", &self.border_radius)
      .field("req_is_elevated", &self.req_is_elevated)
      .field("is_elevated", &self.is_elevated)
      .field("icon", &self.icon)
      .field("icon_color", &self.icon_color)
      .field("label", &self.label)
      .field("label_color", &self.label_color)
      .field("child_align", &self.child_align)
      .field("animation_sm", &self.animation_sm)
      .field("mouse_down_position", &self.mouse_down_position)
      .finish_non_exhaustive()
  }
}

impl<'a> State<'a> for ButtonState<'_> {
  fn on_mouse_over(&mut self, _mouse_position: (f32, f32)) -> bool {
    context::HAND_CURSOR.with(|hand_cursor| hand_cursor.set());

    if let Some(on_mouse_over) = &self.on_mouse_over {
      on_mouse_over.lock().unwrap()();
    }

    true
  }

  fn on_mouse_out(&mut self, _mouse_position: (f32, f32)) -> bool {
    context::ARROW_CURSOR.with(|arrow_cursor| arrow_cursor.set());

    self.is_elevated = self.req_is_elevated;
    self.animation_sm.fade_out();

    if let Some(on_mouse_out) = &self.on_mouse_out {
      on_mouse_out.lock().unwrap()();
    }

    true
  }

  fn on_mouse_down(&mut self, mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if mouse_button != MouseButton::Left {
      return true;
    }

    self.is_elevated = false;
    self.mouse_down_position = mouse_position;
    self.animation_sm.ripple();

    if let Some(on_mouse_down) = &self.on_mouse_down {
      on_mouse_down.lock().unwrap()();
    }

    true
  }

  fn on_mouse_up(&mut self, _mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if mouse_button != MouseButton::Left {
      return true;
    }

    self.is_elevated = self.req_is_elevated;
    self.animation_sm.fade_out();

    if let Some(on_mouse_up) = &self.on_mouse_up {
      on_mouse_up.lock().unwrap()();
    }

    true
  }

  fn update(&mut self, dt: f32) -> bool {
    self.animation_sm.update(dt)
  }

  fn build(&mut self, constraint: Rect) -> Widget<'a> {
    self.animation_sm.constraint = constraint;

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
            constraint.x() + constraint.width() * 0.5,
            constraint.y() + constraint.height() * 0.5,
          ),
          size: (0.0, 0.0),
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
                        .font_size(28.0)
                        .font_style(FontStyle::new(
                          Weight::SEMI_BOLD,
                          Width::NORMAL,
                          Slant::Upright,
                        ))
                        .color(self.label_color)
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
        },
      ],
    }
    .into()
  }

  fn post_draw(&self, canvas: &Canvas, constraint: Rect) {
    if self.animation_sm.ripple_radius.get_now() <= 0.0
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
          panic!("Animating while in {state:?} state which is unexpected")
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
      1.0,
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
