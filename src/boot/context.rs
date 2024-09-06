use crate::models::AudioTask;
use atomic_float::AtomicF32;
use skia_safe::{FontMgr, Typeface};
use std::{
  collections::HashMap,
  fs,
  sync::{mpsc::Sender, LazyLock, Mutex, OnceLock},
};

pub static AUDIO_TX: OnceLock<Sender<AudioTask<'_>>> = OnceLock::new();
pub static DRAWABLE_SIZE: (AtomicF32, AtomicF32) = (AtomicF32::new(0.0), AtomicF32::new(0.0));

pub static TEXT_TYPEFACES: LazyLock<Mutex<HashMap<String, Typeface>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

pub static ICON_TYPEFACE: LazyLock<Typeface> = LazyLock::new(|| {
  FontMgr::new()
    .new_from_data(
      &fs::read("assets/fonts/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf").unwrap(),
      None,
    )
    .unwrap()
});
