use crate::models::{AudioReq, FontCfg};
use skia_safe::{Font, FontMgr};
use std::{
  cell::RefCell,
  collections::HashMap,
  sync::{mpsc::Sender, OnceLock},
};

pub static AUDIO_TX: OnceLock<Sender<AudioReq>> = OnceLock::new();

thread_local! {
  pub(crate) static FONT_MGR: FontMgr = FontMgr::new();
  pub(crate) static FONT_CACHE: RefCell<HashMap<FontCfg, Font>> = RefCell::new(HashMap::new());
}
