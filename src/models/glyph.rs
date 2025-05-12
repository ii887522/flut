use super::Rect;
use optarg2chain::optarg_impl;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Glyph {
  pub(super) position: (f32, f32),
  pub(super) size: (f32, f32),
  pub(super) tex_position: (f32, f32, f32),
  pub(super) color: u32,
  pub(super) tex_size: (f32, f32),
  pub(super) pad: (f32, f32),
}

#[optarg_impl]
impl Glyph {
  #[optarg_method(GlyphNewBuilder, call)]
  pub fn new(
    position: (f32, f32),
    size: (f32, f32),
    color: (u8, u8, u8, u8),
    #[optarg((-size.0, -size.1, 0.0))] tex_position: (f32, f32, f32),
  ) -> Self {
    Self {
      position,
      size,
      tex_position,
      color: crate::pack_color(color),
      tex_size: size,
      pad: (0.0, 0.0),
    }
  }
}

impl From<Rect> for Glyph {
  fn from(rect: Rect) -> Self {
    Self {
      position: rect.position,
      size: rect.size,
      tex_position: (-rect.size.0, -rect.size.1, 0.0),
      color: rect.color,
      tex_size: rect.size,
      pad: (0.0, 0.0),
    }
  }
}
