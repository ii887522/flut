use crate::{consts, models::Direction};
use flut::models::Rect;

#[derive(Clone, Copy)]
pub(crate) struct Worm {
  pub(crate) position: u32,
  pub(crate) drawable_id: u32,
  pub(crate) direction: Direction,
}

impl Worm {
  pub(crate) const fn calc_next_position(&self) -> u32 {
    match self.direction {
      Direction::Up => self.position - consts::GRID_SIZE.0,
      Direction::Right => self.position + 1,
      Direction::Down => self.position + consts::GRID_SIZE.0,
      Direction::Left => self.position - 1,
    }
  }
}

impl From<Worm> for Rect {
  fn from(worm: Worm) -> Self {
    Self::new()
      .position((
        (worm.position % consts::GRID_SIZE.0 * (consts::CELL_SIZE.0 + consts::GAP_SIZE.0)) as _,
        (worm.position / consts::GRID_SIZE.0 * (consts::CELL_SIZE.1 + consts::GAP_SIZE.1)
          + consts::HEADER_HEIGHT) as _,
      ))
      .size((consts::CELL_SIZE.0 as _, consts::CELL_SIZE.1 as _))
      .color((243, 125, 121, 255))
      .call()
  }
}
