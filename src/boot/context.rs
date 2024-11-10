use crate::models::FontCfg;
use skia_safe::{Font, FontMgr};
use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(crate) static FONT_MGR: FontMgr = FontMgr::new();
  pub(crate) static FONT_CACHE: RefCell<HashMap<FontCfg, Font>> = RefCell::new(HashMap::new());
}
