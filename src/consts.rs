// App loop settings
pub(super) const UPDATES_PER_SECOND: f32 = 120.0;
pub(super) const MAX_FRAME_UPDATE_COUNT: u32 = 8;

// Renderer settings
pub(super) const MAX_IN_FLIGHT_FRAME_COUNT: usize = 3;
pub(super) const SUB_DYNAMIC_BUFFER_COUNT: usize = 2;

// Text related settings
pub(super) const FONT_SIZE: u16 = 48;
pub(super) const GLYPH_ATLAS_SIZE: (u32, u32) = (256, 256);
pub(super) const GLYPH_GAP: u32 = 1;

// Icon related settings
pub(super) const ICON_ATLAS_SIZE: (u32, u32) = (512, 512);
pub(super) const ICON_GAP: u32 = 1;
pub(super) const ICON_COL_COUNT: u32 = ICON_ATLAS_SIZE.0 / (FONT_SIZE as u32 + ICON_GAP);
pub(super) const ICON_ROW_COUNT: u32 = ICON_ATLAS_SIZE.1 / (FONT_SIZE as u32 + ICON_GAP);
pub(super) const ICON_COUNT: u32 = ICON_COL_COUNT * ICON_ROW_COUNT;
