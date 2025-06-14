use crate::consts;
use flut::models::Rect;

#[derive(Clone, Copy)]
pub(crate) struct Air {
  pub(crate) position: u16,
  pub(crate) drawable_id: u16,
}

impl From<Air> for Rect {
  fn from(air: Air) -> Self {
    Self::new(
      (
        (consts::WORLD_POSITION.0
          + (air.position % consts::GRID_SIZE.0) * (consts::CELL_SIZE.0 + consts::GAP_SIZE.0))
          as _,
        (consts::WORLD_POSITION.1
          + (air.position / consts::GRID_SIZE.0) * (consts::CELL_SIZE.1 + consts::GAP_SIZE.1))
          as _,
        1.0,
      ),
      (consts::CELL_SIZE.0 as _, consts::CELL_SIZE.1 as _),
      (48, 48, 48, 255),
    )
  }
}
