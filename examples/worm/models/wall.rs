use crate::consts;
use flut::models::Rect;

#[derive(Clone, Copy)]
pub(crate) struct Wall {
  pub(crate) position: u32,
  pub(crate) drawable_id: u32,
}

impl From<Wall> for Rect {
  fn from(wall: Wall) -> Self {
    Self::new()
      .position((
        (wall.position % consts::GRID_SIZE.0 * (consts::CELL_SIZE.0 + consts::GAP_SIZE.0)) as _,
        (wall.position / consts::GRID_SIZE.0 * (consts::CELL_SIZE.1 + consts::GAP_SIZE.1)
          + consts::HEADER_HEIGHT) as _,
      ))
      .size((consts::CELL_SIZE.0 as _, consts::CELL_SIZE.1 as _))
      .color((255, 0, 0, 255))
      .call()
  }
}
