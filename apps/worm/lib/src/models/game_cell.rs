use crate::models::GameCellType;
use flut::renderers::renderer_ref;

#[derive(Clone, Copy)]
pub(crate) struct GameCell {
  pub(crate) ty: GameCellType,
  pub(crate) render_id: renderer_ref::Id,
}
