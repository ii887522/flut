use super::{
  stateful_widget::State, widget::*, RectWidget, Stack, StackChild, StatefulWidget, Widget,
};
use crate::{
  boot::context,
  helpers::Animation,
  models::{icon_name, Origin},
  widgets::{Button, Icon, Text},
};
use skia_safe::{
  font_style::{Slant, Weight, Width},
  Color, FontStyle, Rect,
};
use std::{
  fmt::{self, Debug, Formatter},
  sync::{atomic::Ordering, Arc, Mutex},
};

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
    // Start pop up dialog animation
    context::ANIMATION_COUNT.fetch_add(1, Ordering::Relaxed);

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
      background_alpha: Animation::new(0.0, 128.0, 0.125),
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
  background_alpha: Animation<f32>,
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
      .field("background_alpha", &self.background_alpha)
      .finish_non_exhaustive()
  }
}

impl<'a> State<'a> for DialogState<'a> {
  fn update(&mut self, dt: f32) -> bool {
    let is_dirty = self.background_alpha.update(dt);

    if self.background_alpha.is_just_ended() {
      // Pop up dialog animation done
      context::ANIMATION_COUNT.fetch_sub(1, Ordering::Relaxed);
    }

    is_dirty
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    // _constraint is unused since this dialog will cover the whole app

    const SIZE: (f32, f32) = (512.0, 256.0);
    const BUTTON_SIZE: (f32, f32) = (208.0, 64.0);
    const BUTTON_GAP: f32 = 32.0;

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
        Some(StackChild {
          position: (0.0, 0.0),
          size: drawable_size,
          origin: Origin::TopLeft,
          child: Some(
            RectWidget {
              color: Color::from_argb(self.background_alpha.get_now() as _, 0, 0, 0),
              ..Default::default()
            }
            .into_widget(),
          ),
        }),
        // Foreground
        Some(StackChild {
          position,
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
            position: (position.0 + 16.0, position.1 + 16.0),
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
            position: (position.0 + 88.0, position.1 + 32.0),
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
          position: (position.0 + 16.0, position.1 + 88.0),
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
            position.1 + SIZE.1 - 80.0,
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
              position.1 + SIZE.1 - 80.0,
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
}
