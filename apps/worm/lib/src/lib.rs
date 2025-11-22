#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

use flut::{Event, models::Rect, renderers::RendererRef};

pub struct Game;

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub fn init(_game: &mut Game, mut renderer: RendererRef<'_>) {
  renderer.add_rect(Rect {
    position: (0.0, 0.0),
    size: (800.0, 450.0),
    color: (0.0, 1.0, 0.0, 1.0),
  });
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub fn process_event(_game: &mut Game, _event: Event) {
  // todo
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub fn update(_game: &mut Game, _dt: f32, _renderer: RendererRef<'_>) {
  // todo
}
