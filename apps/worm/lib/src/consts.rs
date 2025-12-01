// General settings
pub(super) const MIN_SEQ_LEN: usize = 256;
pub const WINDOW_SIZE: (u32, u32) = (1280, 720);
pub(super) const UPDATES_PER_SECOND: f32 = 30.0;

// Grid settings
pub(super) const GRID_SIZE: (f32, f32) = (684.0, 684.0);
pub(super) const GRID_CELL_SIZE: (f32, f32) = (12.0, 12.0);
pub(super) const GRID_CELL_MARGIN: (f32, f32) = (2.0, 2.0);

// Computed grid settings
pub(super) const GRID_POSITION: (f32, f32) = (
  (WINDOW_SIZE.0 as f32 - GRID_SIZE.0) * 0.5,
  (WINDOW_SIZE.1 as f32 - GRID_SIZE.1) * 0.5,
);
pub(super) const GRID_CELL_COUNTS: (u16, u16) = (
  ((GRID_SIZE.0 + GRID_CELL_MARGIN.0) / (GRID_CELL_SIZE.0 + GRID_CELL_MARGIN.0)) as _,
  ((GRID_SIZE.1 + GRID_CELL_MARGIN.1) / (GRID_CELL_SIZE.1 + GRID_CELL_MARGIN.1)) as _,
);
pub(super) const TOTAL_GRID_CELL_COUNT: u16 = GRID_CELL_COUNTS.0 * GRID_CELL_COUNTS.1;
