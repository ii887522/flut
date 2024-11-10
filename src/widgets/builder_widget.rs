use super::Widget;
use sdl2::event::Event;
use skia_safe::{Canvas, Rect};

pub trait BuilderWidget<'a> {
  fn get_size(&self) -> (f32, f32) {
    (-1.0, -1.0)
  }

  fn process_event(&mut self, _event: Event) {}

  fn update(&mut self, _dt: f32) -> bool {
    false
  }

  fn pre_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
  fn build(&mut self, constraint: Rect) -> Widget<'a>;
  fn post_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
}
