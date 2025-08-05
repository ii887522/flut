// App loop settings
pub(super) const UPDATES_PER_SECOND: f32 = 120.0;
pub(super) const MAX_FRAME_UPDATE_COUNT: u32 = 8;

// Renderer settings
pub(super) const MAX_IN_FLIGHT_FRAME_COUNT: usize = 3;

// Text related settings
pub(super) const FONT_SIZE: u16 = 48;
pub(super) const GLYPH_ATLAS_SIZE: (u32, u32) = (256, 256);
pub(super) const GLYPH_GAP: u32 = 1;
