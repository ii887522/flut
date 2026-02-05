pub mod model_capacities;
pub(super) mod push_consts;
pub mod range;
pub mod round_rect;

use crate::{model_sync::ModelSync, renderer::Renderer};

pub trait Model {
  fn get_vertex_count() -> usize;

  fn get_sync<State>(renderer: &mut Renderer<State>) -> &mut ModelSync<Self>
  where
    Self: Sized;
}
