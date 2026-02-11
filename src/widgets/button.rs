use crate::{models::round_rect::RoundRect, renderer_ref::RendererRef, sdf, utils};
use optarg2chain::optarg_impl;
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
  color: u32,
  round_rect_render_id: u32,
  scale: f32,
  state: State,
}

#[optarg_impl]
impl Button {
  #[optarg_method(ButtonNewBuilder, call)]
  pub fn new(
    #[optarg((96.0, 48.0))] size: (f32, f32),
    #[optarg(8.0)] radius: f32,
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
  ) -> Self {
    let (red, green, blue, alpha) = color;

    Self {
      size,
      radius,
      color: utils::pack_color(red, green, blue, alpha),
      round_rect_render_id: u32::MAX,
      scale: 1.0,
      state: State::Initial,
    }
  }

  pub fn init(&mut self, mut renderer: RendererRef<'_>) {
    let (width, height) = self.size;
    let (app_width, app_height) = renderer.get_size();
    let position = ((app_width - width) * 0.5, (app_height - height) * 0.5, 0.0);

    self.round_rect_render_id = renderer.add_model(RoundRect {
      position,
      radius: self.radius,
      size: self.size,
      color: self.color,
    });
  }

  pub fn on_cursor_moved(&mut self, cursor_position: (f32, f32), renderer: &RendererRef<'_>) {
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

    self.state = state;
    let window = renderer.get_window();
    window.set_cursor(Cursor::Icon(cursor_icon));
  }

  pub const fn on_mouse_input(&mut self, input_state: ElementState) {
    self.state = match self.state {
      State::Initial => State::Initial,
      State::Hovered | State::Pressed => match input_state {
        ElementState::Pressed => State::Pressed,
        ElementState::Released => State::Hovered,
      },
    };
  }

  pub fn update(&mut self, dt: f32, renderer: &mut RendererRef<'_>) {
    const SCALE_SPEED: f32 = 2.0;
    const MIN_SCALE: f32 = 0.9;

    let old_scale = self.scale;
    let (width, height) = self.size;
    let (app_width, app_height) = renderer.get_size();

    match self.state {
      State::Initial | State::Hovered => {
        self.scale = SCALE_SPEED.mul_add(dt, self.scale).min(1.0);
        let (width, height) = (width * self.scale, height * self.scale);
        let position = ((app_width - width) * 0.5, (app_height - height) * 0.5, 0.0);

        if self.scale > old_scale {
          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position,
              radius: self.radius * self.scale,
              size: (width, height),
              color: self.color,
            },
          );
        }
      }
      State::Pressed => {
        self.scale = SCALE_SPEED.mul_add(-dt, self.scale).max(MIN_SCALE);
        let (width, height) = (width * self.scale, height * self.scale);
        let position = ((app_width - width) * 0.5, (app_height - height) * 0.5, 0.0);

        if self.scale < old_scale {
          renderer.update_model(
            self.round_rect_render_id,
            RoundRect {
              position,
              radius: self.radius * self.scale,
              size: (width, height),
              color: self.color,
            },
          );
        }
      }
    }
  }
}
