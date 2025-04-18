use super::Glyph;

#[derive(Clone, Copy)]
pub struct Rect {
  pub(super) position: (f32, f32),
  pub(super) size: (f32, f32),
  pub(super) color: u32,
}

impl Rect {
  pub const fn new(position: (f32, f32), size: (f32, f32), color: (u8, u8, u8)) -> Self {
    Self {
      position,
      size,
      color: crate::pack_color(color),
    }
  }
}

impl From<Glyph> for Rect {
  fn from(glyph: Glyph) -> Self {
    Self {
      position: glyph.position,
      size: glyph.size,
      color: glyph.color,
    }
  }
}
