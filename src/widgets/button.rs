use crate::{
  models::round_rect::RoundRect, renderer_ref::RendererRef, sdf, utils, widgets::ripple::Ripple,
};
use optarg2chain::optarg_impl;
use std::collections::VecDeque;
use winit::{
  event::ElementState,
  window::{Cursor, CursorIcon},
};

enum State {
  Initial,
  Hovered,
  Pressed,
}

pub struct Button {
  size: (f32, f32),
  radius: f32,
  color: (u8, u8, u8, u8),
  text_color: (u8, u8, u8, u8),
  on_click: Option<Box<dyn FnMut()>>,
  round_rect_render_id: u32,
  ripples: VecDeque<Ripple>,
  scale: f32,
  color_scale: f32,
  cursor_position: (f32, f32),
  state: State,
}

#[optarg_impl]
impl Button {
  #[optarg_method(ButtonNewBuilder, call)]
  pub fn new(
    #[optarg((96.0, 48.0))] size: (f32, f32),
    #[optarg(8.0)] radius: f32,
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    #[optarg((0, 0, 0, 255))] text_color: (u8, u8, u8, u8),
    #[optarg_default] on_click: Option<Box<dyn FnMut()>>,
  ) -> Self {
    Self {
      size,
      radius,
      color,
      text_color,
      on_click,
      round_rect_render_id: u32::MAX,
      ripples: VecDeque::new(),
      scale: 1.0,
      color_scale: 1.0,
      cursor_position: (0.0, 0.0),
      state: State::Initial,
    }
  }

  pub fn init(&mut self, renderer: &mut RendererRef<'_>) {
    let (width, height) = self.size;
    let (app_width, app_height) = renderer.get_size();
    let position = ((app_width - width) * 0.5, (app_height - height) * 0.5, 0.0);

    self.round_rect_render_id = renderer.add_model(
      RoundRect {
        position,
        radius: self.radius,
        size: self.size,
        color: utils::pack_color(self.color),
      },
      false,
    );
  }

  pub fn on_cursor_moved(&mut self, cursor_position: (f32, f32), renderer: &mut RendererRef<'_>) {
    let (width, height) = self.size;
    let (cursor_x, cursor_y) = cursor_position;
    let (app_width, app_height) = renderer.get_size();
    let (x, y) = ((app_width - width) * 0.5, (app_height - height) * 0.5);

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
    } else if matches!(self.state, State::Pressed) {
      (State::Pressed, CursorIcon::Pointer)
    } else {
      (State::Hovered, CursorIcon::Pointer)
    };

    self.cursor_position = cursor_position;
    self.state = state;
    let window = renderer.get_window();
    window.set_cursor(Cursor::Icon(cursor_icon));
  }

  pub fn on_mouse_input(&mut self, input_state: ElementState, renderer: &mut RendererRef<'_>) {
    self.state = match self.state {
      State::Initial => State::Initial,
      State::Hovered => match input_state {
        ElementState::Pressed => State::Pressed,
        ElementState::Released => State::Hovered,
      },
      State::Pressed => match input_state {
        ElementState::Pressed => State::Pressed,
        ElementState::Released => {
          let (cursor_x, cursor_y) = self.cursor_position;
          let (width, height) = self.size;

          let mut ripple = Ripple::new()
            .position((cursor_x, cursor_y, 0.0))
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
    };
  }

  pub fn update(&mut self, dt: f32, renderer: &mut RendererRef<'_>) {
    // Settings
    const SCALE_SPEED: f32 = 2.0;
    const MIN_SCALE: f32 = 0.9;
    const COLOR_SCALE_SPEED: f32 = 2.0;
    const MIN_COLOR_SCALE: f32 = 0.8;

    let old_scale = self.scale;
    let old_color_scale = self.color_scale;
    let (width, height) = self.size;
    let (app_width, app_height) = renderer.get_size();

    let color = match self.state {
      State::Initial => {
        self.scale = SCALE_SPEED.mul_add(dt, self.scale).min(1.0);
        self.color_scale = COLOR_SCALE_SPEED.mul_add(dt, self.color_scale).min(1.0);
        let color = Self::scale_color(self.color, self.color_scale);

        if self.scale > old_scale || self.color_scale > old_color_scale {
          let (width, height) = (width * self.scale, height * self.scale);
          let position = ((app_width - width) * 0.5, (app_height - height) * 0.5, 0.0);

          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position,
              radius: self.radius * self.scale,
              size: (width, height),
              color: utils::pack_color(color),
            },
            false,
          );
        }

        color
      }
      State::Hovered => {
        self.scale = SCALE_SPEED.mul_add(dt, self.scale).min(1.0);
        self.color_scale = COLOR_SCALE_SPEED
          .mul_add(-dt, self.color_scale)
          .max(MIN_COLOR_SCALE);
        let color = Self::scale_color(self.color, self.color_scale);

        if self.scale > old_scale || self.color_scale < old_color_scale {
          let (width, height) = (width * self.scale, height * self.scale);
          let position = ((app_width - width) * 0.5, (app_height - height) * 0.5, 0.0);

          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position,
              radius: self.radius * self.scale,
              size: (width, height),
              color: utils::pack_color(color),
            },
            false,
          );
        }

        color
      }
      State::Pressed => {
        self.scale = SCALE_SPEED.mul_add(-dt, self.scale).max(MIN_SCALE);
        self.color_scale = COLOR_SCALE_SPEED
          .mul_add(-dt, self.color_scale)
          .max(MIN_COLOR_SCALE);
        let color = Self::scale_color(self.color, self.color_scale);

        if self.scale < old_scale || self.color_scale < old_color_scale {
          let (width, height) = (width * self.scale, height * self.scale);
          let position = ((app_width - width) * 0.5, (app_height - height) * 0.5, 0.0);

          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position,
              radius: self.radius * self.scale,
              size: (width, height),
              color: utils::pack_color(color),
            },
            false,
          );
        }

        color
      }
    };

    self.ripples.iter_mut().for_each(|ripple| {
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
