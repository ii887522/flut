use optarg2chain::optarg_impl;

#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct Rect {
  pub(crate) tex_position: (f32, f32, f32),
  pub(crate) color: u32,
  pub(crate) tex_size: (f32, f32),
  pub(crate) position: (f32, f32),
  pub(crate) size: (f32, f32),
  pad: (f32, f32),
}

#[optarg_impl]
impl Rect {
  #[optarg_method(RectNewBuilder, call)]
  pub fn new(
    #[optarg_default] position: (f32, f32),
    #[optarg_default] size: (f32, f32),
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    #[optarg((0.5 / crate::consts::GLYPH_ATLAS_SIZE.0 as f32, 0.5 / crate::consts::GLYPH_ATLAS_SIZE.1 as f32, 0.0))]
    tex_position: (f32, f32, f32),
    #[optarg_default] tex_size: (f32, f32),
  ) -> Self {
    Self {
      tex_position,
      color: crate::pack_color(color),
      tex_size,
      position,
      size,
      pad: (0.0, 0.0),
    }
  }
}
