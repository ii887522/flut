use super::Widget;
use sdl2::event::Event;
use skia_safe::{Canvas, Rect};
use std::fmt::Debug;

pub trait StatefulWidget<'a>: Debug + Send {
  fn get_size(&self) -> (f32, f32) {
    (0.0, 0.0)
  }

  fn new_state(&self) -> Box<dyn State<'a> + 'a>;
}

pub trait State<'a>: Debug {
  fn process_event(&mut self, _event: &Event) {}

  fn update(&mut self, _dt: f32) -> bool {
    false
  }

  fn pre_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
  fn build(&self, constraint: Rect) -> Widget<'a>;
  fn post_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
}
