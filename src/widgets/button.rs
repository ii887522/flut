use crate::{
  glyph_renderer::{IconId, TextId},
  models::{align::Align, font_key::FontKey, icon::Icon, round_rect::RoundRect, text::Text},
  renderer_ref::RendererRef,
  sdf, utils,
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
const TEXT_Z_OFFSET: f32 = -0.0001;
const ICON_MARGIN: f32 = 16.0;

// Event handlers
type OnClick = dyn FnMut();
type OnMouseInput = dyn FnMut(ElementState, MouseButton);
type OnMouseMoved = dyn FnMut((f32, f32));
type OnMouseEnter = dyn FnMut();
type OnMouseLeave = dyn FnMut();

enum State {
  Initial,
  Hovered,
  LeftPressed,
  RightPressed,
}

pub struct Button {
  old_position: (f32, f32, f32),
  position: (f32, f32, f32),
  size: (f32, f32),
  radius: f32,
  color: (u8, u8, u8, u8),
  text_color: (u8, u8, u8, u8),
  icon_font_path: Cow<'static, str>,
  icon_codepoint: u16,
  old_text: Cow<'static, str>,
  text: Cow<'static, str>,
  on_click: Option<Box<OnClick>>,
  on_mouse_input: Option<Box<OnMouseInput>>,
  on_mouse_moved: Option<Box<OnMouseMoved>>,
  on_mouse_enter: Option<Box<OnMouseEnter>>,
  on_mouse_leave: Option<Box<OnMouseLeave>>,
  round_rect_render_id: u32,
  ripples: VecDeque<Ripple>,
  icon_render_id: Option<IconId>,
  text_render_id: Option<TextId>,
  scale: f32,
  color_scale: f32,
  old_mouse_position: (f32, f32),
  mouse_position: (f32, f32),
  state: State,
}

#[optarg_impl]
impl Button {
  #[optarg_method(ButtonNewBuilder, call)]
  pub fn new(
    #[optarg_default] position: (f32, f32, f32),
    #[optarg((80.0, 40.0))] size: (f32, f32),
    #[optarg(8.0)] radius: f32,
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    #[optarg((0, 0, 0, 255))] text_color: (u8, u8, u8, u8),
    #[optarg_default] icon_font_path: Cow<'static, str>,
    #[optarg_default] icon_codepoint: u16,
    #[optarg_default] text: Cow<'static, str>,
  ) -> Self {
    Self {
      old_position: position,
      position,
      size,
      radius,
      color,
      text_color,
      icon_font_path,
      icon_codepoint,
      old_text: text.clone(),
      text,
      on_click: None,
      on_mouse_input: None,
      on_mouse_moved: None,
      on_mouse_enter: None,
      on_mouse_leave: None,
      round_rect_render_id: u32::MAX,
      ripples: VecDeque::new(),
      icon_render_id: None,
      text_render_id: None,
      scale: 1.0,
      color_scale: 1.0,
      old_mouse_position: (f32::MIN, f32::MIN),
      mouse_position: (f32::MIN, f32::MIN),
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
  pub fn set_on_mouse_moved(&mut self, on_mouse_moved: Box<OnMouseMoved>) {
    self.on_mouse_moved = Some(on_mouse_moved);
  }

  #[inline]
  pub fn set_on_mouse_enter(&mut self, on_mouse_enter: Box<OnMouseEnter>) {
    self.on_mouse_enter = Some(on_mouse_enter);
  }

  #[inline]
  pub fn set_on_mouse_leave(&mut self, on_mouse_leave: Box<OnMouseLeave>) {
    self.on_mouse_leave = Some(on_mouse_leave);
  }

  pub fn init(&mut self, renderer: &mut RendererRef<'_>) {
    let (width, height) = self.size;
    let (x, y, z) = self.position;

    self.round_rect_render_id = renderer.add_model(
      RoundRect {
        position: (x, y, z),
        radius: self.radius,
        size: self.size,
        color: utils::pack_color(self.color),
      },
      false,
    );

    let (icon_render_id, icon_width, children_width) = if self.icon_codepoint != 0 {
      let icon_render_id = renderer.add_icon(
        Icon {
          position: (0.0, 0.0, 0.0),
          color: utils::pack_color(self.text_color),
          font_size: height * 0.5,
          font_key: FontKey::Path(self.icon_font_path.clone()),
          codepoint: self.icon_codepoint,
        },
        false,
      );

      let (icon_width, _icon_height) = renderer.get_icon_size(&icon_render_id);
      (Some(icon_render_id), icon_width, icon_width)
    } else {
      (None, 0.0, 0.0)
    };

    let children_width = if self.icon_codepoint != 0 && !self.text.is_empty() {
      children_width + ICON_MARGIN
    } else {
      children_width
    };

    let (text_render_id, children_width) = if self.text.is_empty() {
      (None, children_width)
    } else {
      let text_render_id = renderer.add_text(
        &Text {
          position: (0.0, 0.0, 0.0),
          color: utils::pack_color(self.text_color),
          font_size: height * 0.4,
          font_key: FontKey::Family {
            font_family: (&[FamilyName::SansSerif]).into(),
            font_props: Properties {
              weight: Weight::SEMIBOLD,
              ..Default::default()
            },
          },
          align: Align::Left,
          text: self.text.clone(),
        },
        false,
      );

      let (text_width, _text_height) = renderer.get_text_size(&text_render_id);
      (Some(text_render_id), children_width + text_width)
    };

    let child_x = (width - children_width).mul_add(0.5, x);

    let child_x = if let Some(icon_render_id) = icon_render_id {
      let final_icon_render_id = renderer.add_icon(
        Icon {
          position: (child_x, height.mul_add(0.75, y), z + TEXT_Z_OFFSET),
          color: utils::pack_color(self.text_color),
          font_size: height * 0.5,
          font_key: FontKey::Path(self.icon_font_path.clone()),
          codepoint: self.icon_codepoint,
        },
        false,
      );

      renderer.remove_icon(icon_render_id);
      self.icon_render_id = Some(final_icon_render_id);
      child_x + icon_width + ICON_MARGIN
    } else {
      child_x
    };

    if let Some(text_render_id) = text_render_id {
      let final_text_render_id = renderer.add_text(
        &Text {
          position: (child_x, height.mul_add(0.65, y), z + TEXT_Z_OFFSET),
          color: utils::pack_color(self.text_color),
          font_size: height * 0.4,
          font_key: FontKey::Family {
            font_family: (&[FamilyName::SansSerif]).into(),
            font_props: Properties {
              weight: Weight::SEMIBOLD,
              ..Default::default()
            },
          },
          align: Align::Left,
          text: self.text.clone(),
        },
        false,
      );

      renderer.remove_text(text_render_id);
      self.text_render_id = Some(final_text_render_id);
    }
  }

  #[inline]
  pub const fn set_position(&mut self, position: (f32, f32, f32)) {
    self.position = position;
  }

  #[inline]
  pub fn set_text(&mut self, text: Cow<'static, str>) {
    self.text = text;
  }

  pub fn on_mouse_moved(&mut self, mouse_position: (f32, f32), renderer: &mut RendererRef<'_>) {
    let (width, height) = self.size;
    let (old_mouse_x, old_mouse_y) = self.old_mouse_position;
    let (x, y, _) = self.position;

    let sd = sdf::sd_round_rect(
      (
        width.mul_add(-0.5, old_mouse_x - x),
        height.mul_add(-0.5, old_mouse_y - y),
      ),
      (width * 0.5, height * 0.5),
      self.radius,
    );

    let state = if sd > 0.0 {
      State::Initial
    } else if matches!(self.state, State::LeftPressed) {
      State::LeftPressed
    } else if matches!(self.state, State::RightPressed) {
      State::RightPressed
    } else {
      State::Hovered
    };

    if matches!(self.state, State::Initial) && !matches!(state, State::Initial) {
      let window = renderer.get_window();
      window.set_cursor(Cursor::Icon(CursorIcon::Pointer));

      if let Some(ref mut on_mouse_enter) = self.on_mouse_enter {
        on_mouse_enter();
      }
    }

    if !matches!(self.state, State::Initial)
      && let Some(ref mut on_mouse_moved) = self.on_mouse_moved
    {
      on_mouse_moved(mouse_position);
    }

    if !matches!(self.state, State::Initial) && matches!(state, State::Initial) {
      let window = renderer.get_window();
      window.set_cursor(Cursor::Icon(CursorIcon::Default));

      if let Some(ref mut on_mouse_leave) = self.on_mouse_leave {
        on_mouse_leave();
      }
    }

    self.mouse_position = mouse_position;
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
          let (mouse_x, mouse_y) = self.mouse_position;
          let (_, _, z) = self.position;
          let (width, height) = self.size;

          let mut ripple = Ripple::new()
            .position((mouse_x, mouse_y, z))
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
    let (old_x, old_y, _) = self.old_position;
    let (x, y, z) = self.position;
    let (width, height) = self.size;

    match self.state {
      State::Initial => {
        self.scale = SCALE_SPEED.mul_add(dt, self.scale).min(1.0);
        self.color_scale = COLOR_SCALE_SPEED.mul_add(dt, self.color_scale).min(1.0);
      }
      State::Hovered => {
        self.scale = SCALE_SPEED.mul_add(dt, self.scale).min(1.0);
        self.color_scale = COLOR_SCALE_SPEED
          .mul_add(-dt, self.color_scale)
          .max(MIN_COLOR_SCALE);
      }
      State::LeftPressed => {
        self.scale = SCALE_SPEED.mul_add(-dt, self.scale).max(MIN_SCALE);
        self.color_scale = COLOR_SCALE_SPEED
          .mul_add(-dt, self.color_scale)
          .max(MIN_COLOR_SCALE);
      }
      State::RightPressed => (),
    }

    let color = Self::scale_color(self.color, self.color_scale);

    if self.position != self.old_position
      || self.scale != old_scale
      || self.color_scale != old_color_scale
      || self.text != self.old_text
    {
      let (scaled_width, scaled_height) = (width * self.scale, height * self.scale);
      let (x, y) = (
        (width - scaled_width).mul_add(0.5, x),
        (height - scaled_height).mul_add(0.5, y),
      );

      renderer.update_model(
        self.round_rect_render_id,
        RoundRect {
          position: (x, y, z),
          radius: self.radius * self.scale,
          size: (scaled_width, scaled_height),
          color: utils::pack_color(color),
        },
        false,
      );

      let (icon_render_id, icon_width, children_width) =
        if let Some(icon_render_id) = self.icon_render_id.take() {
          let final_icon_render_id = renderer.add_icon(
            Icon {
              position: (0.0, 0.0, 0.0),
              color: utils::pack_color(self.text_color),
              font_size: scaled_height * 0.5,
              font_key: FontKey::Path(self.icon_font_path.clone()),
              codepoint: self.icon_codepoint,
            },
            false,
          );

          renderer.remove_icon(icon_render_id);
          let (icon_width, _icon_height) = renderer.get_icon_size(&final_icon_render_id);
          (Some(final_icon_render_id), icon_width, icon_width)
        } else {
          (None, 0.0, 0.0)
        };

      let children_width = if self.icon_codepoint != 0 && !self.text.is_empty() {
        ICON_MARGIN.mul_add(self.scale, children_width)
      } else {
        children_width
      };

      let (text_render_id, children_width) =
        if let Some(text_render_id) = self.text_render_id.take() {
          let final_text_render_id = renderer.add_text(
            &Text {
              position: (0.0, 0.0, 0.0),
              color: utils::pack_color(self.text_color),
              font_size: scaled_height * 0.4,
              font_key: FontKey::Family {
                font_family: (&[FamilyName::SansSerif]).into(),
                font_props: Properties {
                  weight: Weight::SEMIBOLD,
                  ..Default::default()
                },
              },
              align: Align::Left,
              text: self.text.clone(),
            },
            false,
          );

          renderer.remove_text(text_render_id);
          let (text_width, _text_height) = renderer.get_text_size(&final_text_render_id);
          (Some(final_text_render_id), children_width + text_width)
        } else {
          (None, children_width)
        };

      let child_x = (scaled_width - children_width).mul_add(0.5, x);

      let child_x = if let Some(icon_render_id) = icon_render_id {
        let final_icon_render_id = renderer.add_icon(
          Icon {
            position: (child_x, scaled_height.mul_add(0.75, y), z + TEXT_Z_OFFSET),
            color: utils::pack_color(self.text_color),
            font_size: scaled_height * 0.5,
            font_key: FontKey::Path(self.icon_font_path.clone()),
            codepoint: self.icon_codepoint,
          },
          false,
        );

        renderer.remove_icon(icon_render_id);
        self.icon_render_id = Some(final_icon_render_id);
        ICON_MARGIN.mul_add(self.scale, child_x + icon_width)
      } else {
        child_x
      };

      if let Some(text_render_id) = text_render_id {
        let final_text_render_id = renderer.add_text(
          &Text {
            position: (child_x, scaled_height.mul_add(0.65, y), z + TEXT_Z_OFFSET),
            color: utils::pack_color(self.text_color),
            font_size: scaled_height * 0.4,
            font_key: FontKey::Family {
              font_family: (&[FamilyName::SansSerif]).into(),
              font_props: Properties {
                weight: Weight::SEMIBOLD,
                ..Default::default()
              },
            },
            align: Align::Left,
            text: self.text.clone(),
          },
          false,
        );

        renderer.remove_text(text_render_id);
        self.text_render_id = Some(final_text_render_id);
      }
    }

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
    self.old_text = self.text.clone();
    self.old_mouse_position = self.mouse_position;
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
