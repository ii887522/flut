use super::Widget;
use sdl2::event::Event;
use skia_safe::{Canvas, Rect};
use std::fmt::Debug;

pub trait StatefulWidget: Debug + Send {
  fn new_state<'a>(&self) -> Box<dyn State + 'a>;
}

pub trait State: Debug {
  fn process_event(&mut self, _event: &Event) {}

  fn update(&mut self, _dt: f32) -> bool {
    false
  }

  fn pre_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
  fn build<'a>(&self, constraint: Rect) -> Widget<'a>;
  fn post_draw(&self, _canvas: &Canvas, _constraint: Rect) {}
}
