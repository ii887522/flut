/// Signed distance function for rounded rectangle
/// - `position`: point relative to rect center
/// - `half_size`: half of rect dimensions
/// - `radius`: corner radius
#[must_use]
pub fn sd_round_rect(position: (f32, f32), half_size: (f32, f32), radius: f32) -> f32 {
  let (qx, qy) = (
    position.0.abs() - half_size.0 + radius,
    position.1.abs() - half_size.1 + radius,
  );

  let (qx, qy) = (qx.max(0.0), qy.max(0.0));
  (qx * qx + qy * qy).sqrt() - radius
}
