use flut::app::App;
use winit::{
  application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
  window::WindowId,
};

#[derive(Default)]
pub struct Game {
  app: Option<App>,
}

impl ApplicationHandler for Game {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.app.is_none() {
      self.app = Some(App::new(event_loop, "Void", (1280_f64, 720_f64)));
    }
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::RedrawRequested => {
        if let Some(app) = self.app.take() {
          self.app = Some(app.render());
        }
      }
      _ => (),
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(app) = self.app.take() {
      app.drop();
    }
  }
}
