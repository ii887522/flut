use super::{
  stateful_widget::State, widget::*, RectWidget, Stack, StackChild, StatefulWidget,
  StatelessWidget, Widget,
};
use crate::{
  boot::context,
  helpers::{Animation, AnimationCount},
  models::{icon_name, Origin},
  widgets::{Button, Icon, Text},
};
use sdl2::mouse::MouseButton;
use skia_safe::{
  font_style::{Slant, Weight, Width},
  Canvas, Color, Contains, FontStyle, Point, Rect,
};
use std::{
  fmt::{self, Debug, Formatter},
  sync::{atomic::Ordering, Arc, Mutex},
};

const SIZE: (f32, f32) = (512.0, 256.0);

thread_local! {
  static POSITION: (f32, f32) = (
    (context::DRAWABLE_SIZE.0.load(Ordering::Relaxed) - SIZE.0) * 0.5,
    (context::DRAWABLE_SIZE.1.load(Ordering::Relaxed) - SIZE.1) * 0.5,
  );
}

pub struct Dialog<'a> {
  pub color: Color,
  pub header_icon: u16,
  pub header_icon_color: Color,
  pub header_title: String,
  pub header_title_color: Color,
  pub close_icon: u16,
  pub close_label: String,
  pub ok_icon: u16,
  pub ok_label: String,
  pub has_ok: bool,
  pub on_close: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  pub on_ok: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  pub body: Option<Widget<'a>>,
}

impl Debug for Dialog<'_> {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("Dialog")
      .field("color", &self.color)
      .field("header_icon", &self.header_icon)
      .field("header_icon_color", &self.header_icon_color)
      .field("header_title", &self.header_title)
      .field("header_title_color", &self.header_title_color)
      .field("close_icon", &self.close_icon)
      .field("close_label", &self.close_label)
      .field("ok_icon", &self.ok_icon)
      .field("ok_label", &self.ok_label)
      .field("has_ok", &self.has_ok)
      .field("body", &self.body)
      .finish_non_exhaustive()
  }
}

impl Default for Dialog<'_> {
  fn default() -> Self {
    Self {
      color: Color::BLACK,
      header_icon: 0,
      header_icon_color: Color::BLACK,
      header_title: "".to_string(),
      header_title_color: Color::BLACK,
      body: None,
      close_icon: icon_name::CLOSE,
      close_label: "Close".to_string(),
      ok_icon: icon_name::CHECK,
      ok_label: "OK".to_string(),
      has_ok: false,
      on_close: None,
      on_ok: None,
    }
  }
}

impl<'a> StatefulWidget<'a> for Dialog<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(DialogState {
      color: self.color,
      header_icon: self.header_icon,
      header_icon_color: self.header_icon_color,
      header_title: self.header_title.to_string(),
      header_title_color: self.header_title_color,
      close_icon: self.close_icon,
      close_label: self.close_label.to_string(),
      ok_icon: self.ok_icon,
      ok_label: self.ok_label.to_string(),
      has_ok: self.has_ok,
      on_close: self.on_close.take(),
      on_ok: self.on_ok.take(),
      body: self.body.take(),
      animation_sm: DialogAnimationSM::new(),
      is_pressed_outside_dialog: false,
    })
  }

  fn get_size(&self) -> (f32, f32) {
    // (0.0, 0.0) so that this widget can be inserted in Column or Row or any other layout widget.
    // Size is ignored and this widget always cover the whole app
    (0.0, 0.0)
  }
}

struct DialogState<'a> {
  color: Color,
  header_icon: u16,
  header_icon_color: Color,
  header_title: String,
  header_title_color: Color,
  close_icon: u16,
  close_label: String,
  ok_icon: u16,
  ok_label: String,
  has_ok: bool,
  on_close: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  on_ok: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  body: Option<Widget<'a>>,
  animation_sm: DialogAnimationSM,
  is_pressed_outside_dialog: bool,
}

impl Debug for DialogState<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("DialogState")
      .field("color", &self.color)
      .field("header_icon", &self.header_icon)
      .field("header_icon_color", &self.header_icon_color)
      .field("header_title", &self.header_title)
      .field("header_title_color", &self.header_title_color)
      .field("close_icon", &self.close_icon)
      .field("close_label", &self.close_label)
      .field("ok_icon", &self.ok_icon)
      .field("ok_label", &self.ok_label)
      .field("has_ok", &self.has_ok)
      .field("body", &self.body)
      .field("animation_sm", &self.animation_sm)
      .field("is_pressed_outside_dialog", &self.is_pressed_outside_dialog)
      .finish_non_exhaustive()
  }
}

impl<'a> State<'a> for DialogState<'a> {
  fn on_mouse_down(&mut self, mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if mouse_button != MouseButton::Left {
      // Don't consume the event because dialog buttons might need to listen to it
      return false;
    }

    if !POSITION
      .with(|position| Rect::from_xywh(position.0, position.1, SIZE.0, SIZE.1))
      .contains(Point::new(mouse_position.0, mouse_position.1))
    {
      // User pressed outside the dialog
      self.is_pressed_outside_dialog = true;
    }

    // Don't consume the event because dialog buttons might need to listen to it
    false
  }

  fn on_mouse_up(&mut self, mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if mouse_button != MouseButton::Left {
      // Don't consume the event because dialog buttons might need to listen to it
      return false;
    }

    if self.is_pressed_outside_dialog
      && !POSITION
        .with(|position| Rect::from_xywh(position.0, position.1, SIZE.0, SIZE.1))
        .contains(Point::new(mouse_position.0, mouse_position.1))
    {
      // User click outside the dialog
      self.animation_sm.vibrate();
    }

    self.is_pressed_outside_dialog = false;

    // Don't consume the event because dialog buttons might need to listen to it
    false
  }

  fn update(&mut self, dt: f32) -> bool {
    self.animation_sm.update(dt)
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    // _constraint is unused since this dialog will cover the whole app

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
          origin: Origin::TopLeft,
          child: Some(
            RectWidget {
              color: Color::from_argb(self.animation_sm.background_alpha.get_now() as _, 0, 0, 0),
              ..Default::default()
            }
            .into_widget(),
          ),
        },
        // Foreground
        StackChild {
          position: (0.0, 0.0),
          size: drawable_size,
          origin: Origin::TopLeft,
          child: Some(
            DialogInner {
              color: self.color,
              header_icon: self.header_icon,
              header_icon_color: self.header_icon_color,
              header_title: self.header_title.to_string(),
              header_title_color: self.header_title_color,
              close_icon: self.close_icon,
              close_label: self.close_label.to_string(),
              ok_icon: self.ok_icon,
              ok_label: self.ok_label.to_string(),
              has_ok: self.has_ok,
              on_close: self.on_close.as_ref().map(Arc::clone),
              on_ok: self.on_ok.as_ref().map(Arc::clone),
              #[allow(clippy::useless_asref)]
              body: self.body.as_ref().map(Widget::clone),
              scale: self.animation_sm.scale.get_now(),
            }
            .into_widget(),
          ),
        },
      ],
    }
    .into()
  }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DialogAnimationState {
  #[default]
  PopUp,
  Wait,

  // Vibrate states
  ScaleDown,
  ScaleUp,
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct DialogAnimationSM {
  animation_count: AnimationCount,
  background_alpha: Animation<f32>,
  scale: Animation<f32>,
  state: DialogAnimationState,
}

impl DialogAnimationSM {
  fn new() -> Self {
    // Start pop up animation
    let mut animation_count = AnimationCount::new();
    animation_count.incr();

    Self {
      animation_count,
      background_alpha: Animation::new(0.0, 128.0, 0.125),
      scale: Animation::new(0.0, 1.0, 0.125),
      state: DialogAnimationState::PopUp,
    }
  }

  fn update(&mut self, dt: f32) -> bool {
    let is_dirty = self.background_alpha.update(dt) | self.scale.update(dt);

    if self.background_alpha.is_just_ended() || self.scale.is_just_ended() {
      match self.state {
        DialogAnimationState::PopUp | DialogAnimationState::ScaleUp => {
          // PopUp animation done, wait for start vibrate (ScaleDown -> ScaleUp) animation
          self.animation_count = AnimationCount::new();
          self.state = DialogAnimationState::Wait;
        }
        DialogAnimationState::ScaleDown => {
          // Now scale back up to original to simulate vibration
          self.scale = Animation::new(0.95, 1.0, 0.0625);
          self.state = DialogAnimationState::ScaleUp;
        }
        DialogAnimationState::Wait => panic!("Animating while in Wait state which is unexpected"),
      }
    }

    is_dirty
  }

  fn vibrate(&mut self) {
    if self.state != DialogAnimationState::Wait {
      return;
    }

    self.animation_count.incr();
    self.scale = Animation::new(1.0, 0.95, 0.0625);
    self.state = DialogAnimationState::ScaleDown;
  }
}

struct DialogInner<'a> {
  color: Color,
  header_icon: u16,
  header_icon_color: Color,
  header_title: String,
  header_title_color: Color,
  close_icon: u16,
  close_label: String,
  ok_icon: u16,
  ok_label: String,
  has_ok: bool,
  on_close: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  on_ok: Option<Arc<Mutex<dyn FnMut() + 'a + Send>>>,
  body: Option<Widget<'a>>,
  scale: f32,
}

impl Debug for DialogInner<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("DialogInner")
      .field("color", &self.color)
      .field("header_icon", &self.header_icon)
      .field("header_icon_color", &self.header_icon_color)
      .field("header_title", &self.header_title)
      .field("header_title_color", &self.header_title_color)
      .field("close_icon", &self.close_icon)
      .field("close_label", &self.close_label)
      .field("ok_icon", &self.ok_icon)
      .field("ok_label", &self.ok_label)
      .field("has_ok", &self.has_ok)
      .field("body", &self.body)
      .field("scale", &self.scale)
      .finish_non_exhaustive()
  }
}

impl<'a> StatelessWidget<'a> for DialogInner<'a> {
  fn pre_draw(&self, canvas: &Canvas, _constraint: Rect) {
    let drawable_size = (
      context::DRAWABLE_SIZE.0.load(Ordering::Relaxed),
      context::DRAWABLE_SIZE.1.load(Ordering::Relaxed),
    );

    canvas.save();
    canvas.translate((drawable_size.0 * 0.5, drawable_size.1 * 0.5));
    canvas.scale((self.scale, self.scale));
    canvas.translate((-drawable_size.0 * 0.5, -drawable_size.1 * 0.5));
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    // _constraint is unused since this dialog will cover the whole app

    const BUTTON_SIZE: (f32, f32) = (208.0, 64.0);
    const BUTTON_GAP: f32 = 32.0;

    let drawable_size = (
      context::DRAWABLE_SIZE.0.load(Ordering::Relaxed),
      context::DRAWABLE_SIZE.1.load(Ordering::Relaxed),
    );

    Stack {
      children: vec![
        Some(StackChild {
          position: POSITION.with(|position| *position),
          size: SIZE,
          origin: Origin::TopLeft,
          child: Some(
            RectWidget {
              color: self.color,
              border_radius: 8.0,
              ..Default::default()
            }
            .into_widget(),
          ),
        }),
        if self.header_icon == 0 {
          None
        } else {
          Some(StackChild {
            position: POSITION.with(|position| (position.0 + 16.0, position.1 + 16.0)),
            size: (0.0, 0.0),
            origin: Origin::TopLeft,
            child: Some(
              Icon::new(self.header_icon)
                .size(64.0)
                .color(self.header_icon_color)
                .call()
                .into_widget(),
            ),
          })
        },
        if self.header_title.is_empty() {
          None
        } else {
          Some(StackChild {
            position: POSITION.with(|position| (position.0 + 88.0, position.1 + 32.0)),
            size: (0.0, 0.0),
            origin: Origin::TopLeft,
            child: Some(
              Text::new()
                .text(&self.header_title)
                .font_style(FontStyle::new(
                  Weight::SEMI_BOLD,
                  Width::NORMAL,
                  Slant::Upright,
                ))
                .font_size(32.0)
                .color(self.header_title_color)
                .call()
                .into_widget(),
            ),
          })
        },
        self.body.as_ref().map(|body| StackChild {
          position: POSITION.with(|position| (position.0 + 16.0, position.1 + 88.0)),
          size: (SIZE.0 - 32.0, SIZE.1 - 184.0),
          origin: Origin::TopLeft,
          child: Some(Widget::clone(body)),
        }),
        Some(StackChild {
          position: (
            (drawable_size.0
              - BUTTON_SIZE.0
              - if self.has_ok {
                BUTTON_SIZE.0 + BUTTON_GAP
              } else {
                0.0
              })
              * 0.5,
            POSITION.with(|position| position.1) + SIZE.1 - 80.0,
          ),
          size: BUTTON_SIZE,
          origin: Origin::TopLeft,
          child: Some(
            Button {
              bg_color: Color::RED,
              icon: self.close_icon,
              label: self.close_label.to_string(),
              on_mouse_up: self.on_close.as_ref().map(Arc::clone),
              ..Default::default()
            }
            .into_widget(),
          ),
        }),
        if self.has_ok {
          Some(StackChild {
            position: (
              (drawable_size.0 + BUTTON_GAP) * 0.5,
              POSITION.with(|position| position.1) + SIZE.1 - 80.0,
            ),
            size: BUTTON_SIZE,
            origin: Origin::TopLeft,
            child: Some(
              Button {
                bg_color: Color::from_rgb(0, 128, 0),
                icon: self.ok_icon,
                label: self.ok_label.to_string(),
                on_mouse_up: self.on_ok.as_ref().map(Arc::clone),
                ..Default::default()
              }
              .into_widget(),
            ),
          })
        } else {
          None
        },
      ]
      .into_iter()
      .flatten()
      .collect(),
    }
    .into()
  }

  fn post_draw(&self, canvas: &Canvas, _constraint: Rect) {
    canvas.restore();
  }
}
