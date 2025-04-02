use crate::Engine;
use sdl2::event::Event;

pub struct AppConfig {
  pub title: &'static str,
  pub width: u32,
  pub height: u32,
  pub prefer_dgpu: bool,
}

impl Default for AppConfig {
  fn default() -> Self {
    AppConfig {
      title: "",
      width: 1024,
      height: 768,
      prefer_dgpu: false,
    }
  }
}

pub trait App {
  fn get_config(&self) -> AppConfig;
  fn init(&mut self, _engine: &mut Engine<'_>) {}
  fn process_event(&mut self, _event: Event) {}
  fn update(&mut self, _dt: f32, _engine: &mut Engine<'_>) {}
}
