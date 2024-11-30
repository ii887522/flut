use super::PainterWidget;
use crate::{
  boot::context,
  models::{FontCfg, IconName},
};
use optarg2chain::optarg_impl;
use skia_safe::{font::Edging, Canvas, Color, Font, Paint, Point, Rect};

pub struct Icon {
  name: String,
  font_cfg: FontCfg,
  bound: Rect,
  color: Color,
}

#[optarg_impl]
impl Icon {
  #[optarg_method(IconNewBuilder, call)]
  pub fn new(name: IconName, #[optarg(12)] size: u8, #[optarg(Color::BLACK)] color: Color) -> Self {
    context::FONT_CACHE.with_borrow_mut(|font_cache| {
      let font_cfg = FontCfg {
        font_size: size,
        ..Default::default()
      };

      let font = font_cache.entry(font_cfg).or_insert_with(|| {
        context::ICON_TYPEFACE.with(|icon_typeface| Font::new(icon_typeface, size as f32))
      });

      font.set_edging(Edging::AntiAlias);
      let name = String::from_utf16_lossy(&[name as u16]);
      let (_, bound) = font.measure_str(&name, None);

      Self {
        font_cfg,
        name,
        bound,
        color,
      }
    })
  }
}

impl PainterWidget for Icon {
  fn get_size(&self) -> (f32, f32) {
    (self.bound.width(), self.bound.height())
  }

  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    // Uncomment to draw boundary for debugging purpose
    // canvas.draw_rect(
    //   Rect::from_xywh(
    //     constraint.x(),
    //     constraint.y(),
    //     self.bound.width(),
    //     self.bound.height(),
    //   ),
    //   Paint::default()
    //     .set_anti_alias(true)
    //     .set_stroke(true)
    //     .set_stroke_width(2.0)
    //     .set_color(Color::MAGENTA),
    // );

    context::FONT_CACHE.with_borrow(|font_cache| {
      canvas.draw_str(
        &self.name,
        Point::new(
          constraint.x() - self.bound.x(),
          constraint.y() - self.bound.y(),
        ),
        &font_cache[&self.font_cfg],
        Paint::default().set_anti_alias(true).set_color(self.color),
      );
    });
  }
}
