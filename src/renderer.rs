use crate::models::Rect;
use rustc_hash::{FxHashMap, FxHashSet};

pub trait Renderer {
  fn add_rect(&mut self, rect: Rect) -> u32;
  fn add_rects(&mut self, rects: Vec<Rect>) -> Box<[u32]>;
  fn update_rect(&mut self, id: u32, rect: Rect);
  fn update_rects(&mut self, rects: FxHashMap<u32, Rect>);
  fn remove_rect(&mut self, id: u32) -> Option<Rect>;
  fn remove_rects(&mut self, ids: FxHashSet<u32>) -> Box<[(u32, Rect)]>;
}
