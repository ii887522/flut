use crate::{
  models::{align::Align, round_rect::RoundRect, text::Text},
  renderer_ref::RendererRef,
  sdf,
  text_renderer::TextId,
  utils,
  widgets::ripple::Ripple,
};
use font_kit::{
  family_name::FamilyName,
  properties::{Properties, Weight},
};
use optarg2chain::optarg_impl;
use std::{borrow::Cow, collections::VecDeque};
use winit::{
  event::{ElementState, MouseButton},
  window::{Cursor, CursorIcon},
};

// Settings
const SCALE_SPEED: f32 = 2.0;
const MIN_SCALE: f32 = 0.9;
const COLOR_SCALE_SPEED: f32 = 2.0;
const MIN_COLOR_SCALE: f32 = 0.8;
const ROUND_RECT_Z: f32 = 0.1;

// Computed settings
const TEXT_Z: f32 = ROUND_RECT_Z - 0.1;

type OnClick = dyn FnMut();
type OnMouseInput = dyn FnMut(ElementState, MouseButton);
type OnCursorMoved = dyn FnMut((f32, f32));

enum State {
  Initial,
  Hovered,
  LeftPressed,
  RightPressed,
}

pub struct Button {
  old_position: (f32, f32),
  position: (f32, f32),
  size: (f32, f32),
  radius: f32,
  color: (u8, u8, u8, u8),
  text_color: (u8, u8, u8, u8),
  text: Cow<'static, str>,
  on_click: Option<Box<OnClick>>,
  on_mouse_input: Option<Box<OnMouseInput>>,
  on_cursor_moved: Option<Box<OnCursorMoved>>,
  round_rect_render_id: u32,
  ripples: VecDeque<Ripple>,
  text_render_id: Option<TextId>,
  scale: f32,
  color_scale: f32,
  cursor_position: (f32, f32),
  state: State,
}

#[optarg_impl]
impl Button {
  #[optarg_method(ButtonNewBuilder, call)]
  pub fn new(
    #[optarg_default] position: (f32, f32),
    #[optarg((80.0, 40.0))] size: (f32, f32),
    #[optarg(8.0)] radius: f32,
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    #[optarg((0, 0, 0, 255))] text_color: (u8, u8, u8, u8),
    #[optarg_default] text: Cow<'static, str>,
  ) -> Self {
    Self {
      old_position: position,
      position,
      size,
      radius,
      color,
      text_color,
      text,
      on_click: None,
      on_mouse_input: None,
      on_cursor_moved: None,
      round_rect_render_id: u32::MAX,
      ripples: VecDeque::new(),
      text_render_id: None,
      scale: 1.0,
      color_scale: 1.0,
      cursor_position: (0.0, 0.0),
      state: State::Initial,
    }
  }

  #[inline]
  pub fn set_on_click(&mut self, on_click: Box<OnClick>) {
    self.on_click = Some(on_click);
  }

  #[inline]
  pub fn set_on_mouse_input(&mut self, on_mouse_input: Box<OnMouseInput>) {
    self.on_mouse_input = Some(on_mouse_input);
  }

  #[inline]
  pub fn set_on_cursor_moved(&mut self, on_cursor_moved: Box<OnCursorMoved>) {
    self.on_cursor_moved = Some(on_cursor_moved);
  }

  pub fn init(&mut self, renderer: &mut RendererRef<'_>) {
    let (width, height) = self.size;
    let (x, y) = self.position;

    self.round_rect_render_id = renderer.add_model(
      RoundRect {
        position: (x, y, ROUND_RECT_Z),
        radius: self.radius,
        size: self.size,
        color: utils::pack_color(self.color),
      },
      false,
    );

    if !self.text.is_empty() {
      self.text_render_id = Some(renderer.add_text(
        Text {
          position: (width.mul_add(0.5, x), height.mul_add(0.65, y), TEXT_Z),
          color: utils::pack_color(self.text_color),
          font_size: height * 0.4,
          font_family: (&[FamilyName::SansSerif]).into(),
          font_props: Properties {
            weight: Weight::SEMIBOLD,
            ..Default::default()
          },
          align: Align::Center,
          text: self.text.clone(),
        },
        false,
      ));
    }
  }

  #[inline]
  pub const fn set_position(&mut self, position: (f32, f32)) {
    self.position = position;
  }

  #[inline]
  pub fn set_text(&mut self, text: Cow<'static, str>) {
    self.text = text;
  }

  pub fn on_cursor_moved(&mut self, cursor_position: (f32, f32), renderer: &mut RendererRef<'_>) {
    let (width, height) = self.size;
    let (cursor_x, cursor_y) = cursor_position;
    let (x, y) = self.position;

    let sd = sdf::sd_round_rect(
      (
        width.mul_add(-0.5, cursor_x - x),
        height.mul_add(-0.5, cursor_y - y),
      ),
      (width * 0.5, height * 0.5),
      self.radius,
    );

    let (state, cursor_icon) = if sd > 0.0 {
      (State::Initial, CursorIcon::Default)
    } else if matches!(self.state, State::LeftPressed) {
      (State::LeftPressed, CursorIcon::Pointer)
    } else if matches!(self.state, State::RightPressed) {
      (State::RightPressed, CursorIcon::Pointer)
    } else {
      (State::Hovered, CursorIcon::Pointer)
    };

    let window = renderer.get_window();
    window.set_cursor(Cursor::Icon(cursor_icon));

    if !matches!(self.state, State::Initial)
      && let Some(ref mut on_cursor_moved) = self.on_cursor_moved
    {
      on_cursor_moved(cursor_position);
    }

    self.cursor_position = cursor_position;
    self.state = state;
  }

  pub fn on_mouse_input(
    &mut self,
    input_state: ElementState,
    button: MouseButton,
    renderer: &mut RendererRef<'_>,
  ) {
    self.state = match self.state {
      State::Initial => State::Initial,
      State::Hovered => match input_state {
        ElementState::Pressed => match button {
          MouseButton::Left => State::LeftPressed,
          MouseButton::Right => State::RightPressed,
          _ => State::Hovered,
        },
        ElementState::Released => State::Hovered,
      },
      State::LeftPressed => match input_state {
        ElementState::Pressed => State::LeftPressed,
        ElementState::Released => {
          let (cursor_x, cursor_y) = self.cursor_position;
          let (width, height) = self.size;

          let mut ripple = Ripple::new()
            .position((cursor_x, cursor_y, ROUND_RECT_Z))
            .start_color(utils::mix_color(self.color, self.text_color))
            .end_color(self.color)
            .end_radius((width * width + height * height).sqrt())
            .duration(1.25)
            .clipped(true)
            .call();

          ripple.init(renderer);
          self.ripples.push_back(ripple);

          if let Some(ref mut on_click) = self.on_click {
            on_click();
          }

          State::Hovered
        }
      },
      State::RightPressed => match input_state {
        ElementState::Pressed => State::RightPressed,
        ElementState::Released => State::Hovered,
      },
    };

    if !matches!(self.state, State::Initial)
      && let Some(ref mut on_mouse_input) = self.on_mouse_input
    {
      on_mouse_input(input_state, button);
    }
  }

  pub fn update(&mut self, dt: f32, renderer: &mut RendererRef<'_>) {
    let old_scale = self.scale;
    let old_color_scale = self.color_scale;
    let (old_x, old_y) = self.old_position;
    let (x, y) = self.position;
    let (width, height) = self.size;

    let color = match self.state {
      State::Initial => {
        self.scale = SCALE_SPEED.mul_add(dt, self.scale).min(1.0);
        self.color_scale = COLOR_SCALE_SPEED.mul_add(dt, self.color_scale).min(1.0);
        let color = Self::scale_color(self.color, self.color_scale);

        if self.position != self.old_position
          || self.scale > old_scale
          || self.color_scale > old_color_scale
        {
          let (scaled_width, scaled_height) = (width * self.scale, height * self.scale);
          let (x, y) = (
            (width - scaled_width).mul_add(0.5, x),
            (height - scaled_height).mul_add(0.5, y),
          );

          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position: (x, y, ROUND_RECT_Z),
              radius: self.radius * self.scale,
              size: (scaled_width, scaled_height),
              color: utils::pack_color(color),
            },
            false,
          );

          if let Some(text_render_id) = self.text_render_id.take() {
            renderer.remove_text(text_render_id);

            self.text_render_id = Some(renderer.add_text(
              Text {
                position: (
                  scaled_width.mul_add(0.5, x),
                  scaled_height.mul_add(0.65, y),
                  TEXT_Z,
                ),
                color: utils::pack_color(self.text_color),
                font_size: scaled_height * 0.4,
                font_family: (&[FamilyName::SansSerif]).into(),
                font_props: Properties {
                  weight: Weight::SEMIBOLD,
                  ..Default::default()
                },
                align: Align::Center,
                text: self.text.clone(),
              },
              false,
            ));
          }
        }

        color
      }
      State::Hovered => {
        self.scale = SCALE_SPEED.mul_add(dt, self.scale).min(1.0);
        self.color_scale = COLOR_SCALE_SPEED
          .mul_add(-dt, self.color_scale)
          .max(MIN_COLOR_SCALE);
        let color = Self::scale_color(self.color, self.color_scale);

        if self.position != self.old_position
          || self.scale > old_scale
          || self.color_scale < old_color_scale
        {
          let (scaled_width, scaled_height) = (width * self.scale, height * self.scale);
          let (x, y) = (
            (width - scaled_width).mul_add(0.5, x),
            (height - scaled_height).mul_add(0.5, y),
          );

          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position: (x, y, ROUND_RECT_Z),
              radius: self.radius * self.scale,
              size: (scaled_width, scaled_height),
              color: utils::pack_color(color),
            },
            false,
          );

          if let Some(text_render_id) = self.text_render_id.take() {
            renderer.remove_text(text_render_id);

            self.text_render_id = Some(renderer.add_text(
              Text {
                position: (
                  scaled_width.mul_add(0.5, x),
                  scaled_height.mul_add(0.65, y),
                  TEXT_Z,
                ),
                color: utils::pack_color(self.text_color),
                font_size: scaled_height * 0.4,
                font_family: (&[FamilyName::SansSerif]).into(),
                font_props: Properties {
                  weight: Weight::SEMIBOLD,
                  ..Default::default()
                },
                align: Align::Center,
                text: self.text.clone(),
              },
              false,
            ));
          }
        }

        color
      }
      State::LeftPressed => {
        self.scale = SCALE_SPEED.mul_add(-dt, self.scale).max(MIN_SCALE);
        self.color_scale = COLOR_SCALE_SPEED
          .mul_add(-dt, self.color_scale)
          .max(MIN_COLOR_SCALE);
        let color = Self::scale_color(self.color, self.color_scale);

        if self.position != self.old_position
          || self.scale < old_scale
          || self.color_scale < old_color_scale
        {
          let (scaled_width, scaled_height) = (width * self.scale, height * self.scale);

          let (x, y) = (
            (width - scaled_width).mul_add(0.5, x),
            (height - scaled_height).mul_add(0.5, y),
          );

          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position: (x, y, ROUND_RECT_Z),
              radius: self.radius * self.scale,
              size: (scaled_width, scaled_height),
              color: utils::pack_color(color),
            },
            false,
          );

          if let Some(text_render_id) = self.text_render_id.take() {
            renderer.remove_text(text_render_id);

            self.text_render_id = Some(renderer.add_text(
              Text {
                position: (
                  scaled_width.mul_add(0.5, x),
                  scaled_height.mul_add(0.65, y),
                  TEXT_Z,
                ),
                color: utils::pack_color(self.text_color),
                font_size: scaled_height * 0.4,
                font_family: (&[FamilyName::SansSerif]).into(),
                font_props: Properties {
                  weight: Weight::SEMIBOLD,
                  ..Default::default()
                },
                align: Align::Center,
                text: self.text.clone(),
              },
              false,
            ));
          }
        }

        color
      }
      State::RightPressed => {
        let color = Self::scale_color(self.color, self.color_scale);

        if self.position != self.old_position {
          let (scaled_width, scaled_height) = (width * self.scale, height * self.scale);
          let (x, y) = (
            (width - scaled_width).mul_add(0.5, x),
            (height - scaled_height).mul_add(0.5, y),
          );

          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position: (x, y, ROUND_RECT_Z),
              radius: self.radius * self.scale,
              size: (scaled_width, scaled_height),
              color: utils::pack_color(color),
            },
            false,
          );

          if let Some(text_render_id) = self.text_render_id.take() {
            renderer.remove_text(text_render_id);

            self.text_render_id = Some(renderer.add_text(
              Text {
                position: (
                  scaled_width.mul_add(0.5, x),
                  scaled_height.mul_add(0.65, y),
                  TEXT_Z,
                ),
                color: utils::pack_color(self.text_color),
                font_size: scaled_height * 0.4,
                font_family: (&[FamilyName::SansSerif]).into(),
                font_props: Properties {
                  weight: Weight::SEMIBOLD,
                  ..Default::default()
                },
                align: Align::Center,
                text: self.text.clone(),
              },
              false,
            ));
          }
        }

        color
      }
    };

    self.ripples.iter_mut().for_each(|ripple| {
      ripple.translate((x - old_x, y - old_y, 0.0));
      ripple.set_end_color(color);
      ripple.update(dt, renderer);
    });

    while let Some(ripple) = self.ripples.front() {
      if !ripple.is_ended() {
        break;
      }

      let old_ripple = self.ripples.pop_front().unwrap();
      old_ripple.drop(renderer);
    }

    self.old_position = self.position;
  }

  #[inline]
  fn scale_color(color: (u8, u8, u8, u8), scale: f32) -> (u8, u8, u8, u8) {
    let (red, green, blue, alpha) = color;

    (
      Self::scale_color_component(red, scale),
      Self::scale_color_component(green, scale),
      Self::scale_color_component(blue, scale),
      alpha,
    )
  }

  #[inline]
  fn scale_color_component(color: u8, scale: f32) -> u8 {
    f32::from(color).mul_add(scale, if color >= 128 { 0.0 } else { 1.0 - scale }) as u8
  }
}
