use super::{
  button::LabelStyle, stateful_widget::State, widget::*, RectWidget, Stack, StackChild,
  StatefulWidget, StatelessWidget, Widget,
};
use crate::{
  boot::context,
  helpers::{consts, Animation, AnimationCount},
  models::{icon_name, Origin, TextStyle},
  widgets::{button::ButtonIcon, Button, Icon, Text},
};
use sdl2::mouse::MouseButton;
use skia_safe::{
  font_style::{Slant, Weight, Width},
  Canvas, Color, Contains, FontStyle, Point, Rect,
};
use std::{
  borrow::Cow,
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

#[derive(Clone, Debug, PartialEq)]
pub struct DialogHeader {
  pub icon: u16,
  pub icon_color: Color,
  pub title: Cow<'static, str>,
  pub title_style: TitleStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TitleStyle {
  pub font_family: &'static str,
  pub font_style: FontStyle,
  pub font_size: f32,
  pub color: Color,
}

impl Default for TitleStyle {
  fn default() -> Self {
    Self {
      font_family: consts::DEFAULT_FONT_FAMILY,
      font_style: FontStyle::new(Weight::SEMI_BOLD, Width::NORMAL, Slant::Upright),
      font_size: 32.0,
      color: Color::BLACK,
    }
  }
}

impl From<TitleStyle> for TextStyle {
  fn from(style: TitleStyle) -> Self {
    Self {
      font_family: style.font_family,
      font_style: style.font_style,
      font_size: style.font_size,
      color: style.color,
    }
  }
}

impl Default for DialogHeader {
  fn default() -> Self {
    Self {
      icon: 0,
      icon_color: Color::BLACK,
      title: Cow::Borrowed(""),
      title_style: TitleStyle::default(),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogButton {
  pub icon: u16,
  pub icon_color: Color,
  pub color: Color,
  pub label: Cow<'static, str>,
  pub label_style: LabelStyle,
}

impl Default for DialogButton {
  fn default() -> Self {
    Self {
      icon: 0,
      icon_color: Color::BLACK,
      color: Color::BLACK,
      label: Cow::Borrowed(""),
      label_style: LabelStyle::default(),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CloseButton {
  pub icon: u16,
  pub icon_color: Color,
  pub color: Color,
  pub label: Cow<'static, str>,
  pub label_style: LabelStyle,
}

impl Default for CloseButton {
  fn default() -> Self {
    Self {
      icon: icon_name::CLOSE,
      icon_color: Color::BLACK,
      color: Color::RED,
      label: Cow::Borrowed("Close"),
      label_style: LabelStyle::default(),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OkButton {
  pub icon: u16,
  pub icon_color: Color,
  pub color: Color,
  pub label: Cow<'static, str>,
  pub label_style: LabelStyle,
}

impl Default for OkButton {
  fn default() -> Self {
    Self {
      icon: icon_name::CHECK,
      icon_color: Color::BLACK,
      color: Color::from_rgb(0, 128, 0),
      label: Cow::Borrowed("OK"),
      label_style: LabelStyle::default(),
    }
  }
}

pub struct Dialog<'a> {
  pub color: Color,
  pub header: DialogHeader,
  pub close_btn: CloseButton,
  pub ok_btn: OkButton,
  pub has_ok: bool,
  pub on_close: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_ok: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub body: Option<Widget<'a>>,
}

impl Debug for Dialog<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("Dialog")
      .field("color", &self.color)
      .field("header", &self.header)
      .field("close_btn", &self.close_btn)
      .field("ok_btn", &self.ok_btn)
      .field("has_ok", &self.has_ok)
      .field("body", &self.body)
      .finish_non_exhaustive()
  }
}

impl Default for Dialog<'_> {
  fn default() -> Self {
    Self {
      color: Color::BLACK,
      header: DialogHeader::default(),
      close_btn: CloseButton::default(),
      ok_btn: OkButton::default(),
      has_ok: false,
      on_close: Arc::new(Mutex::new(|| {})),
      on_ok: Arc::new(Mutex::new(|| {})),
      body: None,
    }
  }
}

impl<'a> StatefulWidget<'a> for Dialog<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(DialogState {
      color: self.color,
      header: self.header.clone(),
      close_btn: self.close_btn.clone(),
      ok_btn: self.ok_btn.clone(),
      has_ok: self.has_ok,
      on_close: Arc::clone(&self.on_close),
      on_ok: Arc::clone(&self.on_ok),
      #[allow(clippy::useless_asref)]
      body: self.body.as_ref().map(Widget::clone),
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
  header: DialogHeader,
  close_btn: CloseButton,
  ok_btn: OkButton,
  has_ok: bool,
  on_close: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_ok: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  body: Option<Widget<'a>>,
  animation_sm: DialogAnimationSM,
  is_pressed_outside_dialog: bool,
}

impl Debug for DialogState<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("DialogState")
      .field("color", &self.color)
      .field("header", &self.header)
      .field("close_btn", &self.close_btn)
      .field("ok_btn", &self.ok_btn)
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
              header: self.header.clone(),
              close_btn: self.close_btn.clone(),
              ok_btn: self.ok_btn.clone(),
              has_ok: self.has_ok,
              on_close: Arc::clone(&self.on_close),
              on_ok: Arc::clone(&self.on_ok),
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
pub struct DialogAnimationSM {
  animation_count: AnimationCount,
  background_alpha: Animation<f32>,
  scale: Animation<f32>,
  state: DialogAnimationState,
}

impl DialogAnimationSM {
  pub fn new() -> Self {
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
        DialogAnimationState::Wait => {
          unreachable!("Animating while in Wait state which is unexpected");
        }
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
  header: DialogHeader,
  close_btn: CloseButton,
  ok_btn: OkButton,
  has_ok: bool,
  on_close: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_ok: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  body: Option<Widget<'a>>,
  scale: f32,
}

impl Debug for DialogInner<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("DialogInner")
      .field("color", &self.color)
      .field("header", &self.header)
      .field("close_btn", &self.close_btn)
      .field("ok_btn", &self.ok_btn)
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
        if self.header.icon == 0 {
          None
        } else {
          Some(StackChild {
            position: POSITION.with(|position| (position.0 + 44.0, position.1 + 44.0)),
            size: (0.0, 0.0),
            origin: Origin::Center,
            child: Some(
              Icon::new(self.header.icon)
                .size(64.0)
                .color(self.header.icon_color)
                .call()
                .into_widget(),
            ),
          })
        },
        if self.header.title.is_empty() {
          None
        } else {
          Some(StackChild {
            position: POSITION.with(|position| (position.0 + 88.0, position.1 + 44.0)),
            size: (0.0, 0.0),
            origin: Origin::Left,
            child: Some(
              Text::new()
                .text(self.header.title.to_string())
                .style(self.header.title_style)
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
              bg_color: self.close_btn.color,
              icon: ButtonIcon::Icon {
                name: self.close_btn.icon,
                color: self.close_btn.icon_color,
              },
              label: Cow::Owned(self.close_btn.label.to_string()),
              label_style: self.close_btn.label_style,
              on_mouse_up: Arc::clone(&self.on_close),
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
                bg_color: self.ok_btn.color,
                icon: ButtonIcon::Icon {
                  name: self.ok_btn.icon,
                  color: self.ok_btn.icon_color,
                },
                label: Cow::Owned(self.ok_btn.label.to_string()),
                label_style: self.ok_btn.label_style,
                on_mouse_up: Arc::clone(&self.on_ok),
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
