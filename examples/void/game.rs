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
    self.app = Some(App::new(event_loop, "Void", (1280_f64, 720_f64)));
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    if event == WindowEvent::CloseRequested {
      event_loop.exit();
    }
  }
}
