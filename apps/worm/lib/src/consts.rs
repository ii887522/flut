// General settings
pub(super) const MIN_SEQ_LEN: usize = 256;
pub const WINDOW_SIZE: (u32, u32) = (1280, 720);
pub(super) const UPDATES_PER_SECOND: f32 = 28.0;

// Grid settings
pub(super) const GRID_MARGIN: f32 = 56.0;
pub(super) const GRID_SIZE: (f32, f32) = (656.0, 656.0);
pub(super) const GRID_CELL_SIZE: (f32, f32) = (12.0, 12.0);
pub(super) const GRID_CELL_MARGIN: (f32, f32) = (2.0, 2.0);

// Score settings
pub(super) const SCORE_MARGIN: f32 = 44.0;
pub(super) const SCORE_FONT_SIZE: u16 = 48;
pub(super) const SCORE_COLOR: (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 1.0);

// Computed grid settings
pub(super) const GRID_POSITION: (f32, f32) =
  ((WINDOW_SIZE.0 as f32 - GRID_SIZE.0) * 0.5, GRID_MARGIN);
pub(super) const SCORE_POSITION: (f32, f32) = (WINDOW_SIZE.0 as f32 * 0.5, SCORE_MARGIN);
pub(super) const GRID_CELL_COUNTS: (u16, u16) = (
  ((GRID_SIZE.0 + GRID_CELL_MARGIN.0) / (GRID_CELL_SIZE.0 + GRID_CELL_MARGIN.0)) as _,
  ((GRID_SIZE.1 + GRID_CELL_MARGIN.1) / (GRID_CELL_SIZE.1 + GRID_CELL_MARGIN.1)) as _,
);
pub(super) const TOTAL_GRID_CELL_COUNT: u16 = GRID_CELL_COUNTS.0 * GRID_CELL_COUNTS.1;
