use super::{
  stateful_widget::State, widget::IntoPainterWidget, ImageWidget, StatefulWidget, Widget,
};
use sdl2::mouse::MouseButton;
use skia_safe::Rect;
use std::{
  fmt::{self, Debug, Formatter},
  sync::{Arc, Mutex},
};

pub struct ImageButton<'a> {
  pub file_path: &'static str,
  pub size: (f32, f32),
  pub on_mouse_over: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_mouse_out: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_mouse_down: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_mouse_up: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_right_mouse_down: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  pub on_right_mouse_up: Arc<Mutex<dyn FnMut() + 'a + Send>>,
}

impl<'a> Debug for ImageButton<'a> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("ImageButton")
      .field("file_path", &self.file_path)
      .field("size", &self.size)
      .finish_non_exhaustive()
  }
}

impl Default for ImageButton<'_> {
  fn default() -> Self {
    Self {
      file_path: "",
      size: (-1.0, -1.0),
      on_mouse_over: Arc::new(Mutex::new(|| {})),
      on_mouse_out: Arc::new(Mutex::new(|| {})),
      on_mouse_down: Arc::new(Mutex::new(|| {})),
      on_mouse_up: Arc::new(Mutex::new(|| {})),
      on_right_mouse_down: Arc::new(Mutex::new(|| {})),
      on_right_mouse_up: Arc::new(Mutex::new(|| {})),
    }
  }
}

impl<'a> StatefulWidget<'a> for ImageButton<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(ImageButtonState {
      file_path: self.file_path,
      size: self.size,
      on_mouse_over: Arc::clone(&self.on_mouse_over),
      on_mouse_out: Arc::clone(&self.on_mouse_out),
      on_mouse_down: Arc::clone(&self.on_mouse_down),
      on_mouse_up: Arc::clone(&self.on_mouse_up),
      on_right_mouse_down: Arc::clone(&self.on_right_mouse_down),
      on_right_mouse_up: Arc::clone(&self.on_right_mouse_up),
    })
  }
}

struct ImageButtonState<'a> {
  file_path: &'static str,
  size: (f32, f32),
  on_mouse_over: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_mouse_out: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_mouse_down: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_mouse_up: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_right_mouse_down: Arc<Mutex<dyn FnMut() + 'a + Send>>,
  on_right_mouse_up: Arc<Mutex<dyn FnMut() + 'a + Send>>,
}

impl Debug for ImageButtonState<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("ImageButtonState")
      .field("file_path", &self.file_path)
      .field("size", &self.size)
      .finish_non_exhaustive()
  }
}

impl<'a> State<'a> for ImageButtonState<'a> {
  fn on_mouse_over(&mut self, _mouse_position: (f32, f32)) -> bool {
    self.on_mouse_over.lock().unwrap()();
    true
  }

  fn on_mouse_out(&mut self, _mouse_position: (f32, f32)) -> bool {
    self.on_mouse_out.lock().unwrap()();
    true
  }

  fn on_mouse_down(&mut self, _mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    match mouse_button {
      MouseButton::Left => self.on_mouse_down.lock().unwrap()(),
      MouseButton::Right => self.on_right_mouse_down.lock().unwrap()(),
      _ => {}
    }

    true
  }

  fn on_mouse_up(&mut self, _mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    match mouse_button {
      MouseButton::Left => self.on_mouse_up.lock().unwrap()(),
      MouseButton::Right => self.on_right_mouse_up.lock().unwrap()(),
      _ => {}
    }

    true
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    ImageWidget::new(self.file_path)
      .size(self.size)
      .call()
      .into_widget()
  }
}
