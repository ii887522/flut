use crate::consts;
use flut::models::Rect;
use std::sync::atomic::{AtomicU16, Ordering};

pub(crate) struct Food {
  pub(crate) position: u16,
  pub(crate) drawable_id: AtomicU16,
}

impl Default for Food {
  fn default() -> Self {
    Self {
      position: u16::MAX,
      drawable_id: AtomicU16::new(u16::MAX),
    }
  }
}

impl Clone for Food {
  fn clone(&self) -> Self {
    Self {
      position: self.position,
      drawable_id: AtomicU16::new(self.drawable_id.load(Ordering::Relaxed)),
    }
  }
}

impl From<Food> for Rect {
  fn from(food: Food) -> Self {
    Self::new(
      (
        (consts::WORLD_POSITION.0
          + (food.position % consts::GRID_SIZE.0) * (consts::CELL_SIZE.0 + consts::GAP_SIZE.0))
          as _,
        (consts::WORLD_POSITION.1
          + (food.position / consts::GRID_SIZE.0) * (consts::CELL_SIZE.1 + consts::GAP_SIZE.1))
          as _,
        1.0,
      ),
      (consts::CELL_SIZE.0 as _, consts::CELL_SIZE.1 as _),
      (0, 255, 0, 255),
    )
  }
}
