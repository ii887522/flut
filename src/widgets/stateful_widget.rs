use super::Widget;
use sdl2::{event::Event, mouse::MouseButton};
use skia_safe::{Canvas, Rect};

pub trait StatefulWidget<'a>: Send {
  fn get_size(&self) -> (f32, f32) {
    (-1.0, -1.0)
  }

  fn new_state(&mut self) -> Box<dyn State<'a> + 'a>;
}

pub trait State<'a> {
  fn on_mouse_over(&mut self, _mouse_position: (f32, f32)) -> bool {
    false
  }

  fn on_mouse_hover(&mut self, _mouse_position: (f32, f32)) -> bool {
    false
  }

  fn on_mouse_out(&mut self, _mouse_position: (f32, f32)) -> bool {
    false
  }

  fn on_mouse_down(&mut self, _mouse_position: (f32, f32), _mouse_button: MouseButton) -> bool {
    false
  }

  fn on_mouse_up(&mut self, _mouse_position: (f32, f32), _mouse_button: MouseButton) -> bool {
    false
  }

  fn process_event(&mut self, _event: &Event) -> bool {
    false
  }

  fn update(&mut self, _dt: f32) -> bool {
    false
  }

  fn pre_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
  fn build(&mut self, constraint: Rect) -> Widget<'a>;
  fn post_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
}
