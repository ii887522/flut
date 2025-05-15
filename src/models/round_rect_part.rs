use super::RoundRect;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub(crate) struct RoundRectPart {
  position: (f32, f32, f32),
  color: u32,
  pad: (f32, f32, f32),
  control_radius: f32,
  size: (f32, f32),
  control_point: (f32, f32),
}

impl RoundRectPart {
  pub(crate) const fn new(
    position: (f32, f32, f32),
    size: (f32, f32),
    color: (u8, u8, u8, u8),
    control_radius: f32,
    control_point: (f32, f32),
  ) -> Self {
    Self {
      position,
      pad: (0.0, 0.0, 0.0),
      size,
      color: crate::pack_color(color),
      control_radius,
      control_point,
    }
  }
}

impl From<RoundRect> for Vec<RoundRectPart> {
  fn from(round_rect: RoundRect) -> Self {
    vec![
      // Top left
      RoundRectPart::new(
        round_rect.position,
        (round_rect.size.0 * 0.5, round_rect.size.1 * 0.5),
        round_rect.color,
        round_rect.border_radius,
        (
          round_rect.position.0 + round_rect.border_radius,
          round_rect.position.1 + round_rect.border_radius,
        ),
      ),
      // Top right
      RoundRectPart::new(
        (
          round_rect.position.0 + round_rect.size.0 * 0.5,
          round_rect.position.1,
          round_rect.position.2,
        ),
        (round_rect.size.0 * 0.5, round_rect.size.1 * 0.5),
        round_rect.color,
        round_rect.border_radius,
        (
          round_rect.position.0 + round_rect.size.0 - round_rect.border_radius,
          round_rect.position.1 + round_rect.border_radius,
        ),
      ),
      // Bottom right
      RoundRectPart::new(
        (
          round_rect.position.0 + round_rect.size.0 * 0.5,
          round_rect.position.1 + round_rect.size.1 * 0.5,
          round_rect.position.2,
        ),
        (round_rect.size.0 * 0.5, round_rect.size.1 * 0.5),
        round_rect.color,
        round_rect.border_radius,
        (
          round_rect.position.0 + round_rect.size.0 - round_rect.border_radius,
          round_rect.position.1 + round_rect.size.1 - round_rect.border_radius,
        ),
      ),
      // Bottom left
      RoundRectPart::new(
        (
          round_rect.position.0,
          round_rect.position.1 + round_rect.size.1 * 0.5,
          round_rect.position.2,
        ),
        (round_rect.size.0 * 0.5, round_rect.size.1 * 0.5),
        round_rect.color,
        round_rect.border_radius,
        (
          round_rect.position.0 + round_rect.border_radius,
          round_rect.position.1 + round_rect.size.1 - round_rect.border_radius,
        ),
      ),
    ]
  }
}
