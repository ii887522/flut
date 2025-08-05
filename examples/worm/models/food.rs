use crate::consts;
use flut::models::Rect;

#[derive(Clone, Copy)]
pub(crate) struct Food {
  pub(crate) position: u32,
  pub(crate) drawable_id: u32,
}

impl From<Food> for Rect {
  fn from(food: Food) -> Self {
    Self::new()
      .position((
        (food.position % consts::GRID_SIZE.0 * (consts::CELL_SIZE.0 + consts::GAP_SIZE.0)) as _,
        (food.position / consts::GRID_SIZE.0 * (consts::CELL_SIZE.1 + consts::GAP_SIZE.1)
          + consts::HEADER_HEIGHT) as _,
      ))
      .size((consts::CELL_SIZE.0 as _, consts::CELL_SIZE.1 as _))
      .color((0, 255, 0, 255))
      .call()
  }
}
