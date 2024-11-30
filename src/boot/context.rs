use crate::models::{AudioReq, FontCfg};
use atomic_float::AtomicF32;
use skia_safe::{Font, FontMgr, Typeface};
use std::{
  cell::RefCell,
  collections::HashMap,
  fs,
  sync::{mpsc::Sender, OnceLock},
};

pub static AUDIO_TX: OnceLock<Sender<AudioReq>> = OnceLock::new();
pub static DRAWABLE_SIZE: (AtomicF32, AtomicF32) = (AtomicF32::new(0.0), AtomicF32::new(0.0));

thread_local! {
  pub(crate) static FONT_MGR: FontMgr = FontMgr::new();
  pub(crate) static FONT_CACHE: RefCell<HashMap<FontCfg, Font>> = RefCell::new(HashMap::new());

  pub(crate) static ICON_TYPEFACE: Typeface = FONT_MGR.with(|font_mgr| {
    let icon_font_data = fs::read("assets/fonts/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf").unwrap();
    font_mgr.new_from_data(&icon_font_data, None).unwrap()
  });
}
