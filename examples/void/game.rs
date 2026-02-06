use flut::{
  app::App,
  models::{model_capacities::ModelCapacities, round_rect::RoundRect},
  utils,
};
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
    if self.app.is_some() {
      return;
    }

    let mut app = App::new(
      event_loop,
      "Void",
      (1280_f64, 720_f64),
      ModelCapacities::default(),
    );

    app.get_renderer().add_model(RoundRect {
      position: (0.0, 0.0, 0.0),
      radius: 32.0,
      size: (64.0, 64.0),
      color: utils::pack_color(255, 255, 255, 255),
    });

    self.app = Some(app);
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
