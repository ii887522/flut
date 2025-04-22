use crate::consts;
use flut::models::Rect;

#[derive(Clone, Copy)]
pub(crate) struct Wall {
  pub(crate) position: u16,
}

impl From<Wall> for Rect {
  fn from(wall: Wall) -> Self {
    Self::new(
      (
        (consts::WORLD_POSITION.0
          + (wall.position % consts::GRID_SIZE.0) * (consts::CELL_SIZE.0 + consts::GAP_SIZE.0))
          as _,
        (consts::WORLD_POSITION.1
          + (wall.position / consts::GRID_SIZE.0) * (consts::CELL_SIZE.1 + consts::GAP_SIZE.1))
          as _,
      ),
      (consts::CELL_SIZE.0 as _, consts::CELL_SIZE.1 as _),
      (255, 0, 0, 255),
    )
  }
}
