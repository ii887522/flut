use super::{Container, Icon, Text};
use crate::{
  Engine, Transition, consts,
  models::{Anchor, IconName},
};
use optarg2chain::optarg_impl;
use sdl2::{event::Event, mouse::MouseButton};
use std::{
  borrow::Cow,
  fmt::{self, Debug, Formatter},
};

// Config
const ICON_MARGIN: (f32, f32) = (16.0, 8.0);
const ICON_SIZE: (f32, f32) = (35.0, 40.0);
const LABEL_MARGIN: (f32, f32) = (12.0, 12.0);

pub(super) struct Button {
  container: Container,
  icon: Icon,
  label: Text,
  scale: Transition,
  anchor: Anchor,
  on_click: Box<dyn FnMut()>,
  mouse_over: bool,
  mouse_down: bool,
}

impl Debug for Button {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("Button")
      .field("container", &self.container)
      .field("icon", &self.icon)
      .field("label", &self.label)
      .field("scale", &self.scale)
      .field("anchor", &self.anchor)
      .field("mouse_over", &self.mouse_over)
      .field("mouse_down", &self.mouse_down)
      .finish_non_exhaustive()
  }
}

#[optarg_impl]
impl Button {
  #[optarg_method(ButtonNewBuilder, call)]
  pub(super) fn new(
    #[optarg_default] position: (f32, f32, f32),
    icon: IconName,
    #[optarg_default] label: Cow<'static, str>,
    #[optarg(26.0)] font_size: f32,
    #[optarg((255, 255, 255, 255))] bg_color: (u8, u8, u8, u8),
    #[optarg((0, 0, 0, 255))] color: (u8, u8, u8, u8),
    #[optarg(Transition::new(1.0, 1.0, 0.001))] scale: Transition,
    #[optarg_default] scale_origin: (f32, f32),
    #[optarg_default] anchor: Anchor,
    #[optarg(Box::new(|| {}))] on_click: Box<dyn FnMut()>,
  ) -> Self {
    Self {
      container: Container::new((0.0, 0.0))
        .border_radius(10.0)
        .color(bg_color)
        .position(position)
        .scale(scale)
        .scale_origin(scale_origin)
        .call(),
      icon: Icon::new(icon)
        .position((
          position.0 + ICON_MARGIN.0,
          position.1 + ICON_MARGIN.1,
          position.2 - 0.01,
        ))
        .size(ICON_SIZE)
        .color(color)
        .scale(scale)
        .scale_origin(scale_origin)
        .call(),
      label: Text::new()
        .position((
          position.0 + ICON_MARGIN.0 + ICON_SIZE.0 + LABEL_MARGIN.0,
          position.1 + LABEL_MARGIN.1,
          position.2 - 0.01,
        ))
        .size(font_size)
        .color(color)
        .text(label)
        .scale(scale)
        .scale_origin(scale_origin)
        .call(),
      scale,
      anchor,
      on_click,
      mouse_over: false,
      mouse_down: false,
    }
  }

  pub(super) fn init(&mut self, engine: &mut Engine) {
    self.label.set_scale(Transition::new(1.0, 1.0, 0.001));
    self.label.init(engine);
    let label_size = engine.get_text_size();

    self.container.set_size((
      ICON_MARGIN.0 + ICON_SIZE.0 + LABEL_MARGIN.0 + label_size.0 + LABEL_MARGIN.0 * 2.0,
      ICON_MARGIN.1 + label_size.1 + LABEL_MARGIN.1 * 1.5,
    ));

    let offset = crate::calc_position_offset(self.anchor, self.container.get_size());
    let container_position = self.container.get_position();
    let icon_position = self.icon.get_position();
    let label_position = self.label.get_position();

    self.container.set_position((
      container_position.0 + offset.0,
      container_position.1 + offset.1,
      container_position.2,
    ));

    self.icon.set_position((
      icon_position.0 + offset.0,
      icon_position.1 + offset.1,
      icon_position.2,
    ));

    self.label.set_position((
      label_position.0 + offset.0,
      label_position.1 + offset.1,
      label_position.2,
    ));

    self.label.set_scale(self.scale);
    self.container.init(engine);
    self.icon.init(engine);
  }

  pub(super) fn set_scale(&mut self, scale: Transition) {
    self.icon.set_scale(scale);
    self.container.set_scale(scale);
    self.label.set_scale(scale);
  }

  fn is_mouse_on_this(&self, mouse_position: (i32, i32)) -> bool {
    let position = self.container.get_position();
    let size = self.container.get_size();

    mouse_position.0 >= position.0 as _
      && mouse_position.0 < (position.0 + size.0) as _
      && mouse_position.1 >= position.1 as _
      && mouse_position.1 < (position.1 + size.1) as _
  }

  pub(super) fn process_event(&mut self, event: &Event) {
    match event {
      Event::MouseMotion {
        x: mouse_x,
        y: mouse_y,
        ..
      } => {
        if !self.mouse_over && self.is_mouse_on_this((*mouse_x, *mouse_y)) {
          self.mouse_over = true;
          consts::HAND_CURSOR.with(|cursor| cursor.set());
        } else if self.mouse_over && !self.is_mouse_on_this((*mouse_x, *mouse_y)) {
          self.mouse_over = false;
          consts::ARROW_CURSOR.with(|cursor| cursor.set());
        }
      }
      Event::MouseButtonDown {
        mouse_btn,
        x: mouse_x,
        y: mouse_y,
        ..
      } => {
        if *mouse_btn == MouseButton::Left && self.is_mouse_on_this((*mouse_x, *mouse_y)) {
          self.mouse_down = true;
        }
      }
      Event::MouseButtonUp {
        mouse_btn,
        x: mouse_x,
        y: mouse_y,
        ..
      } => {
        if *mouse_btn != MouseButton::Left {
          return;
        }

        if !self.mouse_down || !self.is_mouse_on_this((*mouse_x, *mouse_y)) {
          self.mouse_down = false;
          return;
        }

        self.mouse_down = false;
        (self.on_click)();
      }
      _ => {}
    }
  }

  pub(super) fn update(&mut self, dt: f32, engine: &mut Engine) -> bool {
    let done_scaling = self.icon.update(dt, engine)
      & self.container.update(dt, engine)
      & self.label.update(dt, engine);

    self.label.finish_update(engine);
    done_scaling
  }
}

impl Drop for Button {
  fn drop(&mut self) {
    if self.mouse_over {
      consts::ARROW_CURSOR.with(|cursor| cursor.set());
    }
  }
}
