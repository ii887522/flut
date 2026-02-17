use crate::{model_sync::ModelSync, models::Model, renderer::Renderer};
use std::cmp::Ordering;
use voracious_radix_sort::Radixable;

#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct RoundRect {
  pub position: (f32, f32, f32),
  pub radius: f32,
  pub size: (f32, f32),
  pub color: u32,
}

impl PartialEq for RoundRect {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.position.2 == other.position.2
  }
}

impl PartialOrd for RoundRect {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.position.2.partial_cmp(&other.position.2)
  }
}

impl Radixable<f32> for RoundRect {
  type Key = f32;

  #[inline]
  fn key(&self) -> Self::Key {
    self.position.2
  }
}

impl Model for RoundRect {
  #[inline]
  fn get_vertex_count() -> usize {
    6
  }

  #[inline]
  fn get_sync<State>(renderer: &mut Renderer<State>) -> &mut ModelSync<Self>
  where
    Self: Sized,
  {
    renderer.get_round_rect_sync()
  }

  fn get_clipped_sync<State>(renderer: &mut Renderer<State>) -> &mut ModelSync<Self>
  where
    Self: Sized,
  {
    renderer.get_clipped_round_rect_sync()
  }
}
