use crate::renderer::{Created, Creating, Renderer};
use winit::{
  application::ApplicationHandler,
  event_loop::{ActiveEventLoop, EventLoop},
};

pub fn run<App: ApplicationHandler>(mut app: App) {
  let event_loop = EventLoop::new().unwrap();
  event_loop.run_app(&mut app).unwrap();
}

#[must_use]
pub struct App {
  renderer: Result<Renderer<Created>, Renderer<Creating>>,
}

impl App {
  pub fn new(event_loop: &ActiveEventLoop, title: &str, size: (f64, f64)) -> Self {
    Self {
      renderer: Renderer::new(event_loop, title, size).try_into(),
    }
  }

  pub fn render(self) -> Self {
    Self {
      renderer: match self.renderer {
        Ok(renderer) => renderer.render(),
        Err(renderer) => renderer.try_into(),
      },
    }
  }

  pub fn drop(self) {
    match self.renderer {
      Ok(renderer) => renderer.drop(),
      Err(renderer) => renderer.drop(),
    }
  }
}
