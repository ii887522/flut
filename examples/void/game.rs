use crate::{consts, models::event::Event, widgets::counter_button::CounterButton};
use flut::app::App;
use std::{cell::RefCell, mem, rc::Rc};
use winit::{
  application::ApplicationHandler, dpi::LogicalPosition, event::WindowEvent,
  event_loop::ActiveEventLoop, window::WindowId,
};

pub struct Game {
  app: Option<App>,
  counter_button: CounterButton,
  events: Rc<RefCell<Vec<Event>>>,
}

impl Game {
  #[inline]
  pub(super) fn new() -> Self {
    let events = Rc::new(RefCell::new(vec![]));
    let counter_button = CounterButton::new(&events);

    Self {
      app: None,
      counter_button,
      events,
    }
  }

  fn process_events(&mut self, app: &App) {
    let mut events = self.events.borrow_mut();

    for event in mem::take(&mut *events) {
      self.counter_button.process_event(app, event);
    }
  }
}

impl ApplicationHandler for Game {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.app.is_some() {
      return;
    }

    let mut app = App::new(event_loop)
      .title("Void")
      .size((consts::APP_SIZE.0.into(), consts::APP_SIZE.1.into()))
      .show_fps(true)
      .call();

    let mut renderer = app.get_renderer();
    self.counter_button.init(&mut renderer);
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

        let mut renderer = app.get_renderer();
        let window = renderer.get_window();
        let LogicalPosition { x, y } = cursor_position.to_logical(window.scale_factor());
        self.counter_button.on_cursor_moved((x, y), &mut renderer);
      }
      WindowEvent::MouseInput {
        device_id: _,
        state: input_state,
        button,
      } => {
        let Some(app) = self.app.as_mut() else {
          return;
        };

        let mut renderer = app.get_renderer();

        self
          .counter_button
          .on_mouse_input(input_state, button, &mut renderer);
      }
      WindowEvent::RedrawRequested => {
        let Some(mut app) = self.app.take() else {
          return;
        };

        self.process_events(&app);

        app.update(|dt, renderer| {
          self.counter_button.update(dt, renderer);
        });

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
