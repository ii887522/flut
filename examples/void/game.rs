use crate::{consts, models::event::Event, widgets::counter_button::CounterButton};
use flut::{app::App, widgets::button::Button};
use std::{cell::RefCell, mem, rc::Rc};
use winit::{
  application::ApplicationHandler, dpi::LogicalPosition, event::WindowEvent,
  event_loop::ActiveEventLoop, window::WindowId,
};

// Settings
const SHOP_BUTTON_POSITION: (f32, f32, f32) = (8.0, 8.0, 0.002);

pub struct Game {
  app: Option<App>,
  shop_button: Button,
  counter_button: CounterButton,
  events: Rc<RefCell<Vec<Event>>>,
}

impl Game {
  #[inline]
  pub(super) fn new() -> Self {
    let events = Rc::new(RefCell::new(vec![]));

    let mut shop_button = Button::new()
      .position(SHOP_BUTTON_POSITION)
      .size((112.0, 40.0))
      .text("SHOP")
      .color((0, 0, 255, 255))
      .text_color((255, 255, 255, 255))
      .icon_font_path("assets/void/fonts/MaterialSymbolsOutlined-Regular.ttf")
      .icon_codepoint(consts::ICON_SHOPPING_CART)
      .call();

    let counter_button = CounterButton::new(&events);

    shop_button.set_on_mouse_input({
      let events = Rc::clone(&events);

      Box::new(move |input_state, button| {
        events.borrow_mut().push(Event::MouseInput {
          input_state,
          button,
        });
      })
    });

    Self {
      app: None,
      shop_button,
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
    self.shop_button.init(&mut renderer);
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
        position: mouse_position,
      } => {
        let Some(app) = self.app.as_mut() else {
          return;
        };

        let mut renderer = app.get_renderer();
        let window = renderer.get_window();
        let LogicalPosition { x, y } = mouse_position.to_logical(window.scale_factor());
        self.shop_button.on_mouse_moved((x, y), &mut renderer);
        self.counter_button.on_mouse_moved((x, y), &mut renderer);
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
          .shop_button
          .on_mouse_input(input_state, button, &mut renderer);

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
          self.shop_button.update(dt, renderer);
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
