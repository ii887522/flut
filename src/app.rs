use crate::{
  app_loop::AppLoop,
  models::model_capacities::ModelCapacities,
  renderer::{Created, Creating, Renderer},
  renderer_ref::RendererRef,
};
use optarg2chain::optarg_impl;
use std::borrow::Cow;
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
  app_loop: AppLoop,
  renderer: Result<Renderer<Created>, Renderer<Creating>>,
}

#[optarg_impl]
impl App {
  #[optarg_method(AppNewBuilder, call)]
  pub fn new<'event_loop>(
    event_loop: &'event_loop ActiveEventLoop,
    #[optarg_default] title: Cow<'static, str>,
    #[optarg((800_f64, 600_f64))] size: (f64, f64),
    #[optarg_default] model_capacities: ModelCapacities,
    #[optarg((512, 512))] glyph_atlas_size: (u16, u16),
    #[optarg_default] show_fps: bool,
  ) -> Self {
    Self {
      renderer: Renderer::new(event_loop, &title, size, model_capacities, glyph_atlas_size)
        .try_into(),
      app_loop: AppLoop::new(title, show_fps),
    }
  }

  #[must_use]
  #[inline]
  pub const fn get_renderer(&mut self) -> RendererRef<'_> {
    RendererRef::new(&mut self.renderer)
  }

  pub fn update<OnUpdate: FnMut(f32, &mut RendererRef<'_>)>(&mut self, on_update: OnUpdate) {
    self
      .app_loop
      .update(RendererRef::new(&mut self.renderer), on_update);
  }

  pub fn render(self) -> Self {
    Self {
      app_loop: self.app_loop,
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
