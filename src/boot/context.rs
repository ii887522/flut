use crate::models::AudioTask;
use atomic_float::AtomicF32;
use sdl2::mouse::{Cursor, SystemCursor};
use skia_safe::{FontMgr, Image, Typeface};
use std::{
  cell::{OnceCell, RefCell},
  collections::HashMap,
  fs,
  sync::{mpsc::Sender, LazyLock, RwLock},
};

pub static DRAWABLE_SIZE: (AtomicF32, AtomicF32) = (AtomicF32::new(0.0), AtomicF32::new(0.0));

pub static IMAGES: LazyLock<RwLock<HashMap<&'static str, Image>>> =
  LazyLock::new(|| RwLock::new(HashMap::new()));

thread_local! {
  pub static AUDIO_TX: OnceCell<Sender<AudioTask<'static>>> = const { OnceCell::new() };
  pub(crate) static FONT_MGR: FontMgr = FontMgr::new();

  // Font typefaces
  pub static TEXT_TYPEFACES: RefCell<HashMap<String, Typeface>> = RefCell::new(HashMap::new());
  pub static ICON_TYPEFACE: Typeface = FONT_MGR.with(|font_mgr| {
    let icon_font_data = fs::read("assets/fonts/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf").unwrap();
    font_mgr.new_from_data(&icon_font_data, None).unwrap()
  });

  // Mouse cursors
  pub static ARROW_CURSOR: Cursor = Cursor::from_system(SystemCursor::Arrow).unwrap();
  pub static HAND_CURSOR: Cursor = Cursor::from_system(SystemCursor::Hand).unwrap();
}
