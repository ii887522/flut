use super::PainterWidget;
use skia_safe::{BlurStyle, Canvas, Color, MaskFilter, Paint, RRect, Rect};

#[derive(Clone, Copy, PartialEq)]
pub struct RectWidget {
  pub color: Color,
  pub border_radius: f32,
  pub is_elevated: bool,
}

impl Default for RectWidget {
  fn default() -> Self {
    Self {
      color: Color::BLACK,
      border_radius: 0.0,
      is_elevated: false,
    }
  }
}

impl PainterWidget for RectWidget {
  fn draw(&self, canvas: &Canvas, constraint: Rect) {
    if self.is_elevated {
      // Draw drop shadow
      canvas.draw_rrect(
        RRect::new_rect_xy(
          Rect::from_xywh(
            constraint.x(),
            constraint.y() + 4.0,
            constraint.width(),
            constraint.height(),
          ),
          self.border_radius,
          self.border_radius,
        ),
        Paint::default()
          .set_anti_alias(true)
          .set_color(Color::from_argb(
            128,
            self.color.r() >> 4,
            self.color.g() >> 4,
            self.color.b() >> 4,
          ))
          .set_mask_filter(MaskFilter::blur(BlurStyle::Normal, 3.4, false)),
      );
    }

    // Draw this widget
    canvas.draw_rrect(
      RRect::new_rect_xy(constraint, self.border_radius, self.border_radius),
      Paint::default().set_anti_alias(true).set_color(self.color),
    );
  }
}
