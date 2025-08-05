use optarg2chain::optarg_impl;

#[derive(Clone, Copy)]
#[repr(C, align(8))]
pub struct Rect {
  pub(crate) position: (f32, f32),
  pub(crate) size: (f32, f32),
  pub(crate) tex_position: (f32, f32),
  pub(crate) tex_size: (f32, f32),
  pub(crate) color: u32,
  pad: f32,
}

#[optarg_impl]
impl Rect {
  #[optarg_method(RectNewBuilder, call)]
  pub fn new(
    #[optarg_default] position: (f32, f32),
    #[optarg_default] size: (f32, f32),
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
    #[optarg((0.5, 0.5))] tex_position: (f32, f32),
    #[optarg_default] tex_size: (f32, f32),
  ) -> Self {
    Self {
      position,
      size,
      tex_position,
      tex_size,
      color: crate::pack_color(color),
      pad: 0.0,
    }
  }
}
