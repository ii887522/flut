use crate::consts;

#[inline]
pub(super) const fn to_position(index: u16) -> (f32, f32, u8) {
  (
    (index % consts::GRID_CELL_COUNTS.0) as f32
      * (consts::GRID_CELL_SIZE.0 + consts::GRID_CELL_MARGIN.0)
      + consts::GRID_POSITION.0,
    (index / consts::GRID_CELL_COUNTS.0) as f32
      * (consts::GRID_CELL_SIZE.1 + consts::GRID_CELL_MARGIN.1)
      + consts::GRID_POSITION.1,
    0,
  )
}

#[inline]
pub(super) const fn to_2d_index(index: u16) -> (u16, u16) {
  (
    index % consts::GRID_CELL_COUNTS.0,
    index / consts::GRID_CELL_COUNTS.0,
  )
}
