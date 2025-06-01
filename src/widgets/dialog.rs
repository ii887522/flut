use super::{Button, Container, Glass, Icon, Text};
use crate::{
  Engine, Transition,
  models::{Anchor, IconName},
};
use optarg2chain::optarg_impl;
use sdl2::{event::Event, mouse::MouseButton};
use std::{borrow::Cow, sync::atomic::Ordering};

// Config
const SCALING_UP_DURATION: f32 = 0.125;
const VIBRATE_DURATION: f32 = 0.05;
const MIN_VIBRATE_SCALE: f32 = 0.9;

// Dialog config
const DIALOG_SIZE: (u32, u32) = (512, 256);

// Icon config
const ICON_MARGIN: (f32, f32) = (16.0, 8.0);
const ICON_SIZE: (f32, f32) = (56.0, 64.0);

// Title config
const TITLE_MARGIN: f32 = 18.0;
const TITLE_FONT_SIZE: f32 = 44.0;

// Description config
const DESC_MARGIN: f32 = 16.0;
const DESC_MAX_WIDTH: f32 = DIALOG_SIZE.0 as f32 - ICON_MARGIN.0 * 2.0;

// Button config
const BUTTON_MARGIN: (f32, f32) = (48.0, 16.0);

#[derive(Clone, Copy, Debug)]
enum State {
  ScalingUp,
  Idle,
  ScalingDown,
}

pub struct DialogButton {
  bg_color: (u8, u8, u8, u8),
  color: (u8, u8, u8, u8),
  icon: IconName,
  label: Cow<'static, str>,
  on_click: Box<dyn FnMut()>,
}

#[optarg_impl]
impl DialogButton {
  #[optarg_method(DialogButtonNewBuilder, call)]
  pub fn new(
    #[optarg((0, 0, 0, 255))] bg_color: (u8, u8, u8, u8),
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    icon: IconName,
    #[optarg_default] label: Cow<'static, str>,
    #[optarg(Box::new(|| {}))] on_click: Box<dyn FnMut()>,
  ) -> Self {
    Self {
      bg_color,
      color,
      icon,
      label,
      on_click,
    }
  }
}

#[derive(Debug)]
pub struct Dialog {
  glass: Glass,
  container: Container,
  icon: Icon,
  title: Text,
  desc: Text,
  cancel_button: Button,
  ok_button: Button,
  mouse_down_outside: bool,
  state: State,
}

#[optarg_impl]
impl Dialog {
  #[optarg_method(DialogNewBuilder, call)]
  pub fn new(
    #[optarg((255, 255, 255, 255))] bg_color: (u8, u8, u8, u8),
    #[optarg((0, 0, 0, 255))] color: (u8, u8, u8, u8),
    icon: IconName,
    #[optarg_default] title: Cow<'static, str>,
    #[optarg_default] desc: Cow<'static, str>,
    cancel_button: DialogButton,
    ok_button: DialogButton,
  ) -> Self {
    let app_size = (
      crate::APP_SIZE.0.load(Ordering::Relaxed),
      crate::APP_SIZE.1.load(Ordering::Relaxed),
    );

    let to_position = Self::get_position();

    Self {
      glass: Glass::new((app_size.0 as _, app_size.1 as _))
        .alpha(Transition::new(0.0, 128.0, SCALING_UP_DURATION))
        .call(),
      container: Container::new((DIALOG_SIZE.0 as _, DIALOG_SIZE.1 as _))
        .color(bg_color)
        .position((to_position.0 as _, to_position.1 as _, 1.0))
        .scale(Transition::new(0.0, 1.0, SCALING_UP_DURATION))
        .scale_origin(((app_size.0 >> 1) as _, (app_size.1 >> 1) as _))
        .call(),
      icon: Icon::new(icon)
        .color(color)
        .position((
          to_position.0 as f32 + ICON_MARGIN.0,
          to_position.1 as f32 + ICON_MARGIN.1,
          0.99,
        ))
        .scale(Transition::new(0.0, 1.0, SCALING_UP_DURATION))
        .scale_origin(((app_size.0 >> 1) as _, (app_size.1 >> 1) as _))
        .size(ICON_SIZE)
        .call(),
      title: Text::new()
        .color(color)
        .position((
          to_position.0 as f32 + ICON_MARGIN.0 + ICON_SIZE.0 + TITLE_MARGIN,
          to_position.1 as f32 + TITLE_MARGIN,
          0.99,
        ))
        .scale(Transition::new(0.0, 1.0, SCALING_UP_DURATION))
        .scale_origin(((app_size.0 >> 1) as _, (app_size.1 >> 1) as _))
        .size(TITLE_FONT_SIZE)
        .text(title)
        .call(),
      desc: Text::new()
        .color(color)
        .position((
          to_position.0 as f32 + ICON_MARGIN.0,
          to_position.1 as f32 + ICON_MARGIN.1 + ICON_SIZE.1 + DESC_MARGIN,
          0.99,
        ))
        .max_width(DESC_MAX_WIDTH)
        .scale(Transition::new(0.0, 1.0, SCALING_UP_DURATION))
        .scale_origin(((app_size.0 >> 1) as _, (app_size.1 >> 1) as _))
        .text(desc)
        .call(),
      cancel_button: Button::new(cancel_button.icon)
        .anchor(Anchor::BottomLeft)
        .position((
          to_position.0 as f32 + BUTTON_MARGIN.0,
          to_position.1 as f32 + DIALOG_SIZE.1 as f32 - BUTTON_MARGIN.1,
          0.99,
        ))
        .bg_color(cancel_button.bg_color)
        .color(cancel_button.color)
        .label(cancel_button.label)
        .scale(Transition::new(0.0, 1.0, SCALING_UP_DURATION))
        .scale_origin(((app_size.0 >> 1) as _, (app_size.1 >> 1) as _))
        .on_click(cancel_button.on_click)
        .call(),
      ok_button: Button::new(ok_button.icon)
        .anchor(Anchor::BottomRight)
        .position((
          to_position.0 as f32 + DIALOG_SIZE.0 as f32 - BUTTON_MARGIN.0,
          to_position.1 as f32 + DIALOG_SIZE.1 as f32 - BUTTON_MARGIN.1,
          0.99,
        ))
        .bg_color(ok_button.bg_color)
        .color(ok_button.color)
        .label(ok_button.label)
        .scale(Transition::new(0.0, 1.0, SCALING_UP_DURATION))
        .scale_origin(((app_size.0 >> 1) as _, (app_size.1 >> 1) as _))
        .on_click(ok_button.on_click)
        .call(),
      mouse_down_outside: false,
      state: State::ScalingUp,
    }
  }

  fn get_position() -> (u32, u32) {
    let app_size = (
      crate::APP_SIZE.0.load(Ordering::Relaxed),
      crate::APP_SIZE.1.load(Ordering::Relaxed),
    );

    (
      (app_size.0 - DIALOG_SIZE.0) >> 1,
      (app_size.1 - DIALOG_SIZE.1) >> 1,
    )
  }

  fn is_mouse_on_this(mouse_position: (i32, i32)) -> bool {
    let position = Self::get_position();

    mouse_position.0 >= position.0 as _
      && mouse_position.0 < (position.0 + DIALOG_SIZE.0) as _
      && mouse_position.1 >= position.1 as _
      && mouse_position.1 < (position.1 + DIALOG_SIZE.1) as _
  }

  pub fn init(&mut self, engine: &mut Engine) {
    self.glass.init(engine);
    self.container.init(engine);
    self.icon.init(engine);
    self.title.init(engine);
    self.desc.init(engine);
    self.cancel_button.init(engine);
    self.ok_button.init(engine);
  }

  pub fn process_event(&mut self, event: &Event) {
    self.cancel_button.process_event(event);
    self.ok_button.process_event(event);

    match event {
      Event::MouseButtonDown {
        mouse_btn,
        x: mouse_x,
        y: mouse_y,
        ..
      } => {
        if *mouse_btn == MouseButton::Left && !Self::is_mouse_on_this((*mouse_x, *mouse_y)) {
          self.mouse_down_outside = true;
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

        if !self.mouse_down_outside
          || !matches!(self.state, State::Idle)
          || Self::is_mouse_on_this((*mouse_x, *mouse_y))
        {
          self.mouse_down_outside = false;
          return;
        }

        self.mouse_down_outside = false;
        let to_scale = Transition::new(1.0, MIN_VIBRATE_SCALE, VIBRATE_DURATION);
        self.container.set_scale(to_scale);
        self.icon.set_scale(to_scale);
        self.title.set_scale(to_scale);
        self.desc.set_scale(to_scale);
        self.cancel_button.set_scale(to_scale);
        self.ok_button.set_scale(to_scale);
        self.state = State::ScalingDown;
      }
      _ => {}
    }
  }

  pub fn update(&mut self, dt: f32, engine: &mut Engine) {
    if self.glass.update(dt, engine)
      & self.container.update(dt, engine)
      & self.icon.update(dt, engine)
      & self.title.update(dt, engine)
      & self.desc.update(dt, engine)
      & self.cancel_button.update(dt, engine)
      & self.ok_button.update(dt, engine)
    {
      match self.state {
        State::ScalingUp => self.state = State::Idle,
        State::ScalingDown => {
          let to_scale = Transition::new(MIN_VIBRATE_SCALE, 1.0, VIBRATE_DURATION);
          self.container.set_scale(to_scale);
          self.icon.set_scale(to_scale);
          self.title.set_scale(to_scale);
          self.desc.set_scale(to_scale);
          self.cancel_button.set_scale(to_scale);
          self.ok_button.set_scale(to_scale);
          self.state = State::ScalingUp;
        }
        _ => {}
      }
    }

    self.desc.finish_update(engine);
    self.title.finish_update(engine);
  }
}
