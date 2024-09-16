use super::{
  stateful_widget::State, widget::*, Icon, RectWidget, Row, Spacing, Stack, StackChild,
  StatefulWidget, Text, Widget,
};
use crate::{
  boot::context,
  models::{Origin, VerticalAlign},
};
use sdl2::mouse::MouseButton;
use skia_safe::{
  font_style::{Slant, Weight, Width},
  Color, FontStyle, Rect,
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
      on_mouse_over: None,
      on_mouse_out: None,
      on_mouse_down: None,
      on_mouse_up: None,
    }
  }
}

impl<'a> StatefulWidget<'a> for Button<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(ButtonState {
      bg_color: self.bg_color,
      border_radius: self.border_radius,
      is_elevated: self.is_elevated,
      icon: self.icon,
      icon_color: self.icon_color,
      label: self.label.to_string(),
      label_color: self.label_color,
      on_mouse_over: self.on_mouse_over.take(),
      on_mouse_out: self.on_mouse_out.take(),
      on_mouse_down: self.on_mouse_down.take(),
      on_mouse_up: self.on_mouse_up.take(),
      dirty_count: 0,
    })
  }
}

#[derive(Default)]
struct ButtonState<'a> {
  bg_color: Color,
  border_radius: f32,
  is_elevated: bool,
  icon: u16,
  icon_color: Color,
  label: String,
  label_color: Color,
  on_mouse_over: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  on_mouse_out: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  on_mouse_down: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  on_mouse_up: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  dirty_count: u32,
}

impl Debug for ButtonState<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("ButtonState")
      .field("bg_color", &self.bg_color)
      .field("border_radius", &self.border_radius)
      .field("is_elevated", &self.is_elevated)
      .field("icon", &self.icon)
      .field("icon_color", &self.icon_color)
      .field("label", &self.label)
      .field("label_color", &self.label_color)
      .field("dirty_count", &self.dirty_count)
      .finish_non_exhaustive()
  }
}

impl Drop for ButtonState<'_> {
  fn drop(&mut self) {
    context::ANIMATION_COUNT.fetch_sub(self.dirty_count, Ordering::Relaxed);
  }
}

impl<'a> State<'a> for ButtonState<'_> {
  fn on_mouse_over(&mut self, _mouse_position: (f32, f32)) -> bool {
    context::HAND_CURSOR.with(|hand_cursor| hand_cursor.set());

    if let Some(on_mouse_over) = &mut self.on_mouse_over {
      on_mouse_over.lock().unwrap()();
    }

    true
  }

  fn on_mouse_out(&mut self, _mouse_position: (f32, f32)) -> bool {
    context::ARROW_CURSOR.with(|arrow_cursor| arrow_cursor.set());

    self.is_elevated = true;
    self.dirty_count += 1;
    context::ANIMATION_COUNT.fetch_add(1, Ordering::Relaxed);

    if let Some(on_mouse_out) = &mut self.on_mouse_out {
      on_mouse_out.lock().unwrap()();
    }

    true
  }

  fn on_mouse_down(&mut self, _mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if mouse_button != MouseButton::Left {
      return true;
    }

    self.is_elevated = false;
    self.dirty_count += 1;
    context::ANIMATION_COUNT.fetch_add(1, Ordering::Relaxed);

    if let Some(on_mouse_down) = &mut self.on_mouse_down {
      on_mouse_down.lock().unwrap()();
    }

    true
  }

  fn on_mouse_up(&mut self, _mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if mouse_button != MouseButton::Left {
      return true;
    }

    self.is_elevated = true;
    self.dirty_count += 1;
    context::ANIMATION_COUNT.fetch_add(1, Ordering::Relaxed);

    if let Some(on_mouse_up) = &mut self.on_mouse_up {
      on_mouse_up.lock().unwrap()();
    }

    true
  }

  fn update(&mut self, _dt: f32) -> bool {
    if self.dirty_count > 0 {
      context::ANIMATION_COUNT.fetch_sub(self.dirty_count, Ordering::Relaxed);
      self.dirty_count = 0;
      return true;
    }

    false
  }

  fn build(&mut self, constraint: Rect) -> Widget<'a> {
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
              // Somehow VerticalAlign::Middle looks like align slightly towards the top,
              // have to use VerticalAlign::Bottom as the workaround
              .align(VerticalAlign::Bottom)
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
}
