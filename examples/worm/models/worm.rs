use super::Direction;
use crate::consts;
use flut::models::Rect;
use std::sync::atomic::{AtomicU16, Ordering};

pub(crate) struct Worm {
  pub(crate) position: u16,
  pub(crate) direction: Direction,
  pub(crate) drawable_id: AtomicU16,
}

impl Clone for Worm {
  fn clone(&self) -> Self {
    Self {
      position: self.position,
      direction: self.direction,
      drawable_id: AtomicU16::new(self.drawable_id.load(Ordering::Relaxed)),
    }
  }
}

impl From<Worm> for Rect {
  fn from(worm: Worm) -> Self {
    Self::new(
      (
        (consts::WORLD_POSITION.0
          + (worm.position % consts::GRID_SIZE.0) * (consts::CELL_SIZE.0 + consts::GAP_SIZE.0))
          as _,
        (consts::WORLD_POSITION.1
          + (worm.position / consts::GRID_SIZE.0) * (consts::CELL_SIZE.1 + consts::GAP_SIZE.1))
          as _,
      ),
      (consts::CELL_SIZE.0 as _, consts::CELL_SIZE.1 as _),
      (243, 125, 121, 255),
    )
  }
}
