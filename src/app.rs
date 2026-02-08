use crate::{
  models::model_capacities::ModelCapacities,
  renderer::{Created, Creating, Renderer},
  renderer_ref::RendererRef,
};
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
  pub fn new(
    event_loop: &ActiveEventLoop,
    title: &str,
    size: (f64, f64),
    model_capacities: ModelCapacities,
  ) -> Self {
    Self {
      renderer: Renderer::new(event_loop, title, size, model_capacities).try_into(),
    }
  }

  #[must_use]
  #[inline]
  pub const fn get_renderer(&mut self) -> RendererRef<'_> {
    RendererRef::new(&mut self.renderer)
  }

  pub fn render(self) -> Self {
    Self {
      renderer: match self.renderer {
        Ok(renderer) => renderer.render(),
        Err(renderer) => match Renderer::<Created>::try_from(renderer) {
          Ok(renderer) => renderer.render(),
          Err(renderer) => Err(renderer),
        },
      },
    }
  }

  pub fn request_redraw_if_visible(&self) {
    let window = match self.renderer {
      Ok(ref renderer) => renderer.get_window(),
      Err(ref renderer) => renderer.get_window(),
    };

    if !window.is_minimized().unwrap_or_default() && window.is_visible().unwrap_or(true) {
      window.request_redraw();
    }
  }

  pub fn drop(self) {
    match self.renderer {
      Ok(renderer) => renderer.drop(),
      Err(renderer) => renderer.drop(),
    }
  }
}
