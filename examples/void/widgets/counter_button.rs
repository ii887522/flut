use crate::{consts, models::event::Event};
use flut::{
  app::App, models::audio_req::AudioReq, renderer_ref::RendererRef, widgets::button::Button,
};
use std::{cell::RefCell, rc::Rc};
use winit::event::{ElementState, MouseButton};

// Settings
pub(super) const BUTTON_SIZE: (f32, f32) = (80.0, 40.0);

pub struct CounterButton {
  button: Button,
  button_right_mouse_down: bool,
  count: usize,
}

impl CounterButton {
  pub(crate) fn new(events: &Rc<RefCell<Vec<Event>>>) -> Self {
    let mut button = Button::new()
      .position((
        (consts::APP_SIZE.0 - BUTTON_SIZE.0) * 0.5,
        (consts::APP_SIZE.1 - BUTTON_SIZE.1) * 0.5,
      ))
      .size(BUTTON_SIZE)
      .text("0")
      .call();

    button.set_on_mouse_input({
      let events = Rc::clone(events);

      Box::new(move |input_state, button| {
        events.borrow_mut().push(Event::MouseInput {
          input_state,
          button,
        });
      })
    });

    button.set_on_cursor_moved({
      let events = Rc::clone(events);

      Box::new(move |cursor_position| {
        events
          .borrow_mut()
          .push(Event::CursorMoved { cursor_position });
      })
    });

    button.set_on_click({
      let events = Rc::clone(events);
      Box::new(move || events.borrow_mut().push(Event::Click))
    });

    Self {
      button,
      button_right_mouse_down: false,
      count: 0,
    }
  }

  pub(crate) fn init(&mut self, renderer: &mut RendererRef<'_>) {
    self.button.init(renderer);
  }

  pub(crate) fn on_cursor_moved(
    &mut self,
    cursor_position: (f32, f32),
    renderer: &mut RendererRef<'_>,
  ) {
    self.button.on_cursor_moved(cursor_position, renderer);
  }

  pub(crate) fn on_mouse_input(
    &mut self,
    input_state: ElementState,
    button: MouseButton,
    renderer: &mut RendererRef<'_>,
  ) {
    self.button.on_mouse_input(input_state, button, renderer);
  }

  pub(crate) fn process_event(&mut self, app: &App, event: Event) {
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

  pub(crate) fn update(&mut self, dt: f32, renderer: &mut RendererRef<'_>) {
    self.button.update(dt, renderer);
  }
}
