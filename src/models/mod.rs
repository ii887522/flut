pub mod align;
pub mod audio_req;
pub mod font_key;
pub(super) mod glyph;
pub(super) mod glyph_key;
pub mod icon;
pub mod model_capacities;
pub(super) mod push_consts;
pub mod range;
pub mod round_rect;
pub mod text;

use crate::{model_sync::ModelSync, renderer::Renderer};

pub trait Model {
  fn get_vertex_count() -> usize;

  fn get_sync<State>(renderer: &mut Renderer<State>) -> &mut ModelSync<Self>
  where
    Self: Sized;

  fn get_clipped_sync<State>(renderer: &mut Renderer<State>) -> &mut ModelSync<Self>
  where
    Self: Sized;
}
