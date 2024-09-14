use crate::models::AudioTask;
use atomic_float::AtomicF32;
use sdl2::mouse::{Cursor, SystemCursor};
use skia_safe::{FontMgr, Typeface};
use std::{
  cell::{OnceCell, RefCell},
  collections::HashMap,
  fs,
  sync::{atomic::AtomicU32, mpsc::Sender},
};

pub static DRAWABLE_SIZE: (AtomicF32, AtomicF32) = (AtomicF32::new(0.0), AtomicF32::new(0.0));
pub static ANIMATION_COUNT: AtomicU32 = AtomicU32::new(0);

thread_local! {
  pub static AUDIO_TX: OnceCell<Sender<AudioTask<'static>>> = const { OnceCell::new() };
  pub(crate) static FONT_MGR: FontMgr = FontMgr::new();

  // Font typefaces
  pub static TEXT_TYPEFACES: RefCell<HashMap<String, Typeface>> = RefCell::new(HashMap::new());
  pub static ICON_TYPEFACE: Typeface = FONT_MGR.with(|font_mgr| {
    font_mgr
      .new_from_data(
        &fs::read("assets/fonts/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf").unwrap(),
        None,
      )
      .unwrap()
  });

  // Mouse cursors
  pub static ARROW_CURSOR: Cursor = Cursor::from_system(SystemCursor::Arrow).unwrap();
  pub static HAND_CURSOR: Cursor = Cursor::from_system(SystemCursor::Hand).unwrap();
}
