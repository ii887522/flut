// Settings
pub(super) const APP_SIZE: (u32, u32) = (880, 880);
pub(super) const CELL_SIZE: (u32, u32) = (16, 16);
pub(super) const GAP_SIZE: (u32, u32) = (2, 2);
pub(super) const UPDATES_PER_SECOND: f32 = 30.0;

// Computed
pub(super) const GRID_SIZE: (u32, u32) = (
  (APP_SIZE.0 + GAP_SIZE.0) / (CELL_SIZE.0 + GAP_SIZE.0),
  (APP_SIZE.1 + GAP_SIZE.1) / (CELL_SIZE.1 + GAP_SIZE.1),
);
pub(super) const CELL_COUNT: u32 = GRID_SIZE.0 * GRID_SIZE.1;
pub(super) const INITIAL_WORM_POSITION: u32 = CELL_COUNT >> 1;
