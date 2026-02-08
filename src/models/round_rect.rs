use crate::{model_sync::ModelSync, models::Model, renderer::Renderer};

#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct RoundRect {
  pub position: (f32, f32, f32),
  pub radius: f32,
  pub size: (f32, f32),
  pub color: u32,
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
}
