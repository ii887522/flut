use crate::models::{AssetReq, AudioReq};
use atomic_float::AtomicF32;
use dashmap::DashMap;
use sdl2::{
  event::EventSender,
  mouse::{Cursor, SystemCursor},
};
use skia_safe::{FontMgr, Image, Typeface};
use std::{
  cell::{OnceCell, RefCell},
  collections::HashMap,
  fs,
  sync::{mpsc::Sender, LazyLock, OnceLock},
};

pub static DRAWABLE_SIZE: (AtomicF32, AtomicF32) = (AtomicF32::new(0.0), AtomicF32::new(0.0));
pub static IMAGES: LazyLock<DashMap<&'static str, Image>> = LazyLock::new(DashMap::new);

// Queue producers
pub static EVENT_SENDER: OnceLock<EventSender> = OnceLock::new();
pub static MAIN_ASSET_TX: OnceLock<Sender<AssetReq>> = OnceLock::new();
pub static MAIN_AUDIO_TX: OnceLock<Sender<AudioReq<'static>>> = OnceLock::new();

thread_local! {
  pub static ASSET_TX: OnceCell<Sender<AssetReq>> = const { OnceCell::new() };
  pub(crate) static FONT_MGR: FontMgr = FontMgr::new();

  // Font typefaces
  pub static ICON_TYPEFACE: Typeface = FONT_MGR.with(|font_mgr| {
    let icon_font_data = fs::read("assets/fonts/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf").unwrap();
    font_mgr.new_from_data(&icon_font_data, None).unwrap()
  });
  pub static TEXT_TYPEFACES: RefCell<HashMap<String, Typeface>> = RefCell::new(HashMap::new());

  // Mouse cursors
  pub static ARROW_CURSOR: Cursor = Cursor::from_system(SystemCursor::Arrow).unwrap();
  pub static HAND_CURSOR: Cursor = Cursor::from_system(SystemCursor::Hand).unwrap();
}
