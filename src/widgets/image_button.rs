use super::{
  stateful_widget::State, widget::IntoPainterWidget, ImageWidget, StatefulWidget, Widget,
};
use atomic_refcell::AtomicRefCell;
use sdl2::mouse::MouseButton;
use skia_safe::Rect;
use std::sync::Arc;

pub struct ImageButton<'a> {
  pub is_enabled: bool,
  pub file_path: &'static str,
  pub size: (f32, f32),
  pub on_mouse_over: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  pub on_mouse_out: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  pub on_mouse_down: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  pub on_mouse_up: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  pub on_right_mouse_down: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  pub on_right_mouse_up: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
}

impl Default for ImageButton<'_> {
  fn default() -> Self {
    Self {
      is_enabled: true,
      file_path: "",
      size: (-1.0, -1.0),
      on_mouse_over: Arc::new(AtomicRefCell::new(|| {})),
      on_mouse_out: Arc::new(AtomicRefCell::new(|| {})),
      on_mouse_down: Arc::new(AtomicRefCell::new(|| {})),
      on_mouse_up: Arc::new(AtomicRefCell::new(|| {})),
      on_right_mouse_down: Arc::new(AtomicRefCell::new(|| {})),
      on_right_mouse_up: Arc::new(AtomicRefCell::new(|| {})),
    }
  }
}

impl<'a> StatefulWidget<'a> for ImageButton<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(ImageButtonState {
      is_enabled: self.is_enabled,
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
  is_enabled: bool,
  file_path: &'static str,
  size: (f32, f32),
  on_mouse_over: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  on_mouse_out: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  on_mouse_down: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  on_mouse_up: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  on_right_mouse_down: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
  on_right_mouse_up: Arc<AtomicRefCell<dyn FnMut() + 'a + Send + Sync>>,
}

impl<'a> State<'a> for ImageButtonState<'a> {
  fn on_mouse_over(&mut self, _mouse_position: (f32, f32)) -> bool {
    if !self.is_enabled {
      return true;
    }

    self.on_mouse_over.borrow_mut()();
    true
  }

  fn on_mouse_out(&mut self, _mouse_position: (f32, f32)) -> bool {
    if !self.is_enabled {
      return true;
    }

    self.on_mouse_out.borrow_mut()();
    true
  }

  fn on_mouse_down(&mut self, _mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if !self.is_enabled {
      return true;
    }

    match mouse_button {
      MouseButton::Left => self.on_mouse_down.borrow_mut()(),
      MouseButton::Right => self.on_right_mouse_down.borrow_mut()(),
      _ => {}
    }

    true
  }

  fn on_mouse_up(&mut self, _mouse_position: (f32, f32), mouse_button: MouseButton) -> bool {
    if !self.is_enabled {
      return true;
    }

    match mouse_button {
      MouseButton::Left => self.on_mouse_up.borrow_mut()(),
      MouseButton::Right => self.on_right_mouse_up.borrow_mut()(),
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
