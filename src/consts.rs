use sdl2::{
  mouse::{Cursor, SystemCursor},
  ttf::{self, Sdl2TtfContext},
};
use std::sync::LazyLock;

// Config
pub(super) const UPDATES_PER_SECOND: f32 = 150.0;
pub(super) const MAX_UPDATES_PER_FRAME: u32 = 5;
pub(super) const MAX_IN_FLIGHT_FRAME_COUNT: u32 = 3;
pub(super) const MIN_ALLOC_SIZE: u64 = 4 * 1024 * 1024;
pub(super) const DYNAMIC_IMAGE_COUNT: u32 = 2;
pub(super) const GLYPH_PADDING: u32 = 1;

// Context
pub(super) static TTF: LazyLock<Sdl2TtfContext> = LazyLock::new(|| ttf::init().unwrap());

thread_local! {
  // Mouse cursor
  pub(super) static ARROW_CURSOR: Cursor = Cursor::from_system(SystemCursor::Arrow).unwrap();
  pub(super) static HAND_CURSOR: Cursor = Cursor::from_system(SystemCursor::Hand).unwrap();
}
