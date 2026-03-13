use crate::models::event::Event;
use flut::{app::App, models::audio_req::AudioReq, widgets::button::Button};
use std::{cell::RefCell, mem, rc::Rc};
use winit::{
  application::ApplicationHandler,
  dpi::LogicalPosition,
  event::{ElementState, MouseButton, WindowEvent},
  event_loop::ActiveEventLoop,
  window::WindowId,
};

// Settings
const APP_SIZE: (f32, f32) = (1280.0, 720.0);
const BUTTON_SIZE: (f32, f32) = (80.0, 40.0);

pub struct Game {
  app: Option<App>,
  button: Button,
  button_right_mouse_down: bool,
  count: usize,
  events: Rc<RefCell<Vec<Event>>>,
}

impl Game {
  #[inline]
  pub(super) fn new() -> Self {
    let mut button = Button::new()
      .position((
        (APP_SIZE.0 - BUTTON_SIZE.0) * 0.5,
        (APP_SIZE.1 - BUTTON_SIZE.1) * 0.5,
      ))
      .size(BUTTON_SIZE)
      .text("0")
      .call();

    let events = Rc::new(RefCell::new(vec![]));

    button.set_on_mouse_input({
      let events = Rc::clone(&events);

      Box::new(move |input_state, button| {
        events.borrow_mut().push(Event::MouseInput {
          input_state,
          button,
        });
      })
    });

    button.set_on_cursor_moved({
      let events = Rc::clone(&events);

      Box::new(move |cursor_position| {
        events
          .borrow_mut()
          .push(Event::CursorMoved { cursor_position });
      })
    });

    button.set_on_click({
      let events = Rc::clone(&events);
      Box::new(move || events.borrow_mut().push(Event::Click))
    });

    Self {
      app: None,
      button,
      button_right_mouse_down: false,
      count: 0,
      events,
    }
  }

  fn process_events(&mut self, app: &App) {
    let mut events = self.events.borrow_mut();

    for event in mem::take(&mut *events) {
      match event {
        Event::MouseInput {
          input_state,
          button,
        } => match button {
          MouseButton::Left => match input_state {
            ElementState::Pressed => {
              _ = app
                .get_audio_tx()
                .send(AudioReq::PlaySound("assets/void/audio/mouse_down.mp3"));
            }
            ElementState::Released => {
              _ = app
                .get_audio_tx()
                .send(AudioReq::PlaySound("assets/void/audio/mouse_up.mp3"));
            }
          },
          MouseButton::Right => self.button_right_mouse_down = input_state == ElementState::Pressed,
          _ => (),
        },
        Event::CursorMoved {
          cursor_position: (cursor_x, cursor_y),
        } => {
          if self.button_right_mouse_down {
            self.button.set_position((
              BUTTON_SIZE.0.mul_add(-0.5, cursor_x),
              BUTTON_SIZE.1.mul_add(-0.5, cursor_y),
            ));
          }
        }
        Event::Click => {
          self.count += 1;
          self.button.set_text(self.count.to_string().into());
        }
      }
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
      .size((APP_SIZE.0.into(), APP_SIZE.1.into()))
      .show_fps(true)
      .call();

    let mut renderer = app.get_renderer();
    self.button.init(&mut renderer);
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
        self.button.on_cursor_moved((x, y), &mut renderer);
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
          .button
          .on_mouse_input(input_state, button, &mut renderer);
      }
      WindowEvent::RedrawRequested => {
        let Some(mut app) = self.app.take() else {
          return;
        };

        self.process_events(&app);

        app.update(|dt, renderer| {
          self.button.update(dt, renderer);
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
