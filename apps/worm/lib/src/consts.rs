// General settings
pub(super) const MIN_SEQ_LEN: usize = 256;
pub const WINDOW_SIZE: (u32, u32) = (1280, 720);
pub(super) const UPDATES_PER_SECOND: f32 = 24.0;

// Grid settings
pub(super) const GRID_MARGIN: f32 = 56.0;
pub(super) const GRID_SIZE: (f32, f32) = (656.0, 656.0);
pub(super) const GRID_CELL_SIZE: (f32, f32) = (12.0, 12.0);
pub(super) const GRID_CELL_MARGIN: (f32, f32) = (2.0, 2.0);

// Score settings
pub(super) const SCORE_MARGIN: f32 = 44.0;
pub(super) const SCORE_FONT_SIZE: f32 = 48.0;
pub(super) const SCORE_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 255);

// Dialog settings
pub(super) const DIALOG_SIZE: (f32, f32) = (512.0, 256.0);
pub(super) const DIALOG_COLOR: (u8, u8, u8, u8) = (255, 0, 0, 255);
pub(super) const DIALOG_BORDER_RADIUS: f32 = 16.0;
pub(super) const DIALOG_PADDING: f32 = 16.0;

// Dialog icon settings
pub(super) const DIALOG_ICON_SIZE: f32 = 48.0;
pub(super) const DIALOG_ICON_MARGIN: f32 = 12.0;
pub(super) const DIALOG_ICON_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 255);

// Dialog title settings
pub(super) const DIALOG_TITLE_FONT_SIZE: f32 = 32.0;
pub(super) const DIALOG_TITLE_OFFSET_Y: f32 = 4.0;
pub(super) const DIALOG_TITLE_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 255);
pub(super) const DIALOG_TITLE: &str = "GAME OVER";

// Dialog description settings
pub(super) const DIALOG_DESC_FONT_SIZE: f32 = 24.0;
pub(super) const DIALOG_DESC_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 255);

// Icon names
pub(super) const SENTIMENT_VERY_DISSATISFIED: u16 = 0xe814;

// Computed grid settings
pub(super) const GRID_POSITION: (f32, f32) =
  ((WINDOW_SIZE.0 as f32 - GRID_SIZE.0) * 0.5, GRID_MARGIN);
pub(super) const GRID_CELL_COUNTS: (u16, u16) = (
  ((GRID_SIZE.0 + GRID_CELL_MARGIN.0) / (GRID_CELL_SIZE.0 + GRID_CELL_MARGIN.0)) as _,
  ((GRID_SIZE.1 + GRID_CELL_MARGIN.1) / (GRID_CELL_SIZE.1 + GRID_CELL_MARGIN.1)) as _,
);
pub(super) const TOTAL_GRID_CELL_COUNT: u16 = GRID_CELL_COUNTS.0 * GRID_CELL_COUNTS.1;

// Computed score settings
pub(super) const SCORE_POSITION: (f32, f32, u8) = (WINDOW_SIZE.0 as f32 * 0.5, SCORE_MARGIN, 0);

// Computed dialog settings
pub(super) const DIALOG_POSITION: (f32, f32, u8) = (
  (WINDOW_SIZE.0 as f32 - DIALOG_SIZE.0) * 0.5,
  (WINDOW_SIZE.1 as f32 - DIALOG_SIZE.1) * 0.5,
  0,
);

// Computed dialog icon settings
pub(super) const DIALOG_ICON_POSITION: (f32, f32, u8) = (
  DIALOG_POSITION.0 + DIALOG_PADDING,
  DIALOG_POSITION.1 + DIALOG_PADDING + DIALOG_ICON_SIZE,
  1,
);

// Computed dialog title settings
pub(super) const DIALOG_TITLE_POSITION: (f32, f32, u8) = (
  DIALOG_ICON_POSITION.0 + DIALOG_ICON_SIZE + DIALOG_ICON_MARGIN,
  DIALOG_POSITION.1 + DIALOG_PADDING + DIALOG_TITLE_FONT_SIZE + DIALOG_TITLE_OFFSET_Y,
  1,
);

// Computed dialog description settings
pub(super) const DIALOG_DESC_POSITION: (f32, f32, u8) = (
  DIALOG_POSITION.0 + DIALOG_PADDING,
  DIALOG_ICON_POSITION.1 + DIALOG_ICON_MARGIN + DIALOG_DESC_FONT_SIZE,
  1,
);
