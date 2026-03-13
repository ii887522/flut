use crate::{
  app_loop::AppLoop,
  audio,
  models::{audio_req::AudioReq, model_capacities::ModelCapacities},
  renderer::{Created, Creating, Renderer},
  renderer_ref::RendererRef,
};
use optarg2chain::optarg_impl;
use std::{
  borrow::Cow,
  sync::mpsc::{self, Sender},
  thread,
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
  audio_tx: Sender<AudioReq>,
  renderer: Result<Renderer<Created>, Renderer<Creating>>,
  app_loop: AppLoop,
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
    let (audio_tx, audio_rx) = mpsc::channel();
    thread::spawn(|| audio::main(audio_rx));

    let renderer =
      Renderer::new(event_loop, &title, size, model_capacities, glyph_atlas_size).try_into();

    let app_loop = AppLoop::new(title, show_fps);

    Self {
      audio_tx,
      renderer,
      app_loop,
    }
  }

  #[must_use]
  #[inline]
  pub const fn get_audio_tx(&self) -> &Sender<AudioReq> {
    &self.audio_tx
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
    let renderer = match self.renderer {
      Ok(renderer) => renderer.render(),
      Err(renderer) => match Renderer::<Created>::try_from(renderer) {
        Ok(renderer) => renderer.render(),
        Err(renderer) => Err(renderer),
      },
    };

    Self {
      audio_tx: self.audio_tx,
      renderer,
      app_loop: self.app_loop,
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
