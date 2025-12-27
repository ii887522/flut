use crate::models::RoundRect;

pub fn calc_round_rect_sd(position: (f32, f32), round_rect: RoundRect) -> f32 {
  let round_rect_center = (
    round_rect.position.0 + round_rect.size.0 * 0.5,
    round_rect.position.1 + round_rect.size.1 * 0.5,
  );

  let local_position = (
    position.0 - round_rect_center.0,
    position.1 - round_rect_center.1,
  );

  let round_rect_half_size = (round_rect.size.0 * 0.5, round_rect.size.1 * 0.5);
  let dist_x = (local_position.0.abs() - round_rect_half_size.0 + round_rect.radius).max(0.0);
  let dist_y = (local_position.1.abs() - round_rect_half_size.1 + round_rect.radius).max(0.0);
  let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();
  dist - round_rect.radius
}
