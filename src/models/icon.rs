use crate::models::{GlyphMetrics, IconName, Rect};
use optarg2chain::optarg_impl;

#[derive(Clone, Copy)]
pub struct Icon {
  pub(crate) position: (f32, f32),
  pub(crate) font_size: u16,
  pub(crate) name: IconName,
  pub(crate) color: (u8, u8, u8, u8),
}

#[optarg_impl]
impl Icon {
  #[optarg_method(IconNewBuilder, call)]
  pub fn new(
    #[optarg_default] position: (f32, f32),
    #[optarg(16)] font_size: u16,
    name: IconName,
    #[optarg((255, 255, 255, 255))] color: (u8, u8, u8, u8),
  ) -> Self {
    Self {
      position,
      font_size,
      name,
      color,
    }
  }

  pub(crate) fn into_rect(self, metrics: GlyphMetrics) -> Rect {
    let scale = self.font_size as f32 / crate::consts::FONT_SIZE as f32;

    Rect::new()
      .position(self.position)
      .size((metrics.size.0 * scale, metrics.size.1 * scale))
      .color(self.color)
      .tex_position((
        metrics.position.0 / crate::consts::ICON_ATLAS_SIZE.0 as f32,
        metrics.position.1 / crate::consts::ICON_ATLAS_SIZE.1 as f32,
        metrics.position.2,
      ))
      .tex_size((
        metrics.size.0 / crate::consts::ICON_ATLAS_SIZE.0 as f32,
        metrics.size.1 / crate::consts::ICON_ATLAS_SIZE.1 as f32,
      ))
      .call()
  }
}
