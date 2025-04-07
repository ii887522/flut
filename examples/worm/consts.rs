// Config
pub(super) const APP_SIZE: (u16, u16) = (772, 772);
pub(super) const CELL_SIZE: (u16, u16) = (16, 16);
pub(super) const GAP_SIZE: (u16, u16) = (2, 2);
pub(super) const UPDATES_PER_SECOND: f32 = 30.0;

pub(super) const GRID_SIZE: (u16, u16) = (
  (APP_SIZE.0 + GAP_SIZE.0) / (CELL_SIZE.0 + GAP_SIZE.0),
  (APP_SIZE.1 + GAP_SIZE.1) / (CELL_SIZE.1 + GAP_SIZE.1),
);
pub(super) const INITIAL_WORM_POSITION_2D: (u16, u16) = (GRID_SIZE.0 >> 1, GRID_SIZE.1 >> 1);
pub(super) const INITIAL_WORM_POSITION_1D: u16 =
  INITIAL_WORM_POSITION_2D.0 + INITIAL_WORM_POSITION_2D.1 * GRID_SIZE.0;
