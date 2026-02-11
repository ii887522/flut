use flut::{app::App, models::model_capacities::ModelCapacities, widgets::button::Button};
use winit::{
  application::ApplicationHandler,
  dpi::LogicalPosition,
  event::{MouseButton, WindowEvent},
  event_loop::ActiveEventLoop,
  window::WindowId,
};

// Settings
const APP_SIZE: (f32, f32) = (1280.0, 720.0);

pub struct Game {
  app: Option<App>,
  button: Button,
}

impl Game {
  #[inline]
  pub(super) fn new() -> Self {
    Self {
      app: None,
      button: Button::new().call(),
    }
  }
}

impl ApplicationHandler for Game {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.app.is_some() {
      return;
    }

    let mut app = App::new(
      event_loop,
      "Void".into(),
      (APP_SIZE.0.into(), APP_SIZE.1.into()),
      ModelCapacities::default(),
      true,
    );

    let renderer = app.get_renderer();
    self.button.init(renderer);
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
      WindowEvent::CursorMoved {
        device_id: _,
        position: cursor_position,
      } => {
        let Some(app) = self.app.as_mut() else {
          return;
        };

        let renderer = app.get_renderer();
        let window = renderer.get_window();
        let LogicalPosition { x, y } = cursor_position.to_logical(window.scale_factor());
        self.button.on_cursor_moved((x, y), &renderer);
      }
      WindowEvent::MouseInput {
        device_id: _,
        state: input_state,
        button: MouseButton::Left,
      } => self.button.on_mouse_input(input_state),
      WindowEvent::RedrawRequested => {
        let Some(mut app) = self.app.take() else {
          return;
        };

        app.update(|dt, renderer| self.button.update(dt, renderer));

        let app = app.render();
        app.request_redraw_if_visible();
        self.app = Some(app);
      }
      _ => (),
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(ref app) = self.app {
      app.request_redraw_if_visible();
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(app) = self.app.take() {
      app.drop();
    }
  }
}
