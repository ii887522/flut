use flut::{App, models::Rect, renderers::RendererRef};
use sdl3::event::Event;

pub(super) struct Game;

impl App for Game {
  fn init(&mut self, mut renderer: RendererRef<'_>) {
    renderer.add_rect(Rect {
      position: (0.0, 0.0),
      size: (800.0, 450.0),
      color: (0.0, 1.0, 0.0, 1.0),
    });
  }

  fn process_event(&mut self, _event: Event) {
    //
  }

  fn update(&mut self, _dt: f32, _renderer: RendererRef<'_>) {
    //
  }
}
