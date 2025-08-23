use crate::models::{Icon, Rect, Text};
use rustc_hash::{FxHashMap, FxHashSet};

pub trait Renderer {
  fn set_cam_position(&mut self, cam_position: (f32, f32));
  fn add_rect(&mut self, rect: Rect) -> u32;
  fn add_rects(&mut self, rects: Vec<Rect>) -> Box<[u32]>;
  fn update_rect(&mut self, id: u32, rect: Rect);
  fn update_rects(&mut self, rects: FxHashMap<u32, Rect>);
  fn remove_rect(&mut self, id: u32) -> Option<Rect>;
  fn remove_rects(&mut self, ids: FxHashSet<u32>) -> Box<[(u32, Rect)]>;
  fn add_text(&mut self, text: Text) -> u32;
  fn remove_text(&mut self, id: u32) -> Option<Text>;
  fn add_icon(&mut self, icon: Icon) -> u32;
  fn update_icon(&mut self, id: u32, icon: Icon);
  fn remove_icon(&mut self, id: u32) -> Option<Icon>;
}
