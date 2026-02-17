use crate::models::range::Range;
use voracious_radix_sort::RadixSort as _;

pub fn coalesce_ranges(ranges: &mut Vec<Range>) {
  ranges.voracious_sort();

  let mut last_coalesced_index = 0;
  let mut index = 1;

  while index < ranges.len() {
    let range = ranges[index];
    let last_coalesced_range = &mut ranges[last_coalesced_index];

    if range.start <= last_coalesced_range.end {
      last_coalesced_range.end = last_coalesced_range.end.max(range.end);
    } else {
      last_coalesced_index += 1;
      ranges[last_coalesced_index] = range;
    }

    index += 1;
  }

  ranges.truncate(last_coalesced_index + 1);
}

#[must_use]
#[inline]
pub const fn pack_color(color: (u8, u8, u8, u8)) -> u32 {
  let (red, green, blue, alpha) = color;
  ((red as u32) << 24) | ((green as u32) << 16) | ((blue as u32) << 8) | (alpha as u32)
}

#[must_use]
#[inline]
pub const fn mix_color(color_a: (u8, u8, u8, u8), color_b: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
  let (red_a, green_a, blue_a, alpha_a) = color_a;
  let (red_b, green_b, blue_b, _alpha_b) = color_b;

  (
    (red_a >> 1) + (red_b >> 1),
    (green_a >> 1) + (green_b >> 1),
    (blue_a >> 1) + (blue_b >> 1),
    alpha_a,
  )
}

#[must_use]
#[inline]
pub const fn lerp_color(
  color_a: (u8, u8, u8, u8),
  color_b: (u8, u8, u8, u8),
  scale: f32,
) -> (u8, u8, u8, u8) {
  let (red_a, green_a, blue_a, alpha_a) = color_a;
  let (red_b, green_b, blue_b, alpha_b) = color_b;

  (
    (red_a as f32 + (red_b as f32 - red_a as f32) * scale) as u8,
    (green_a as f32 + (green_b as f32 - green_a as f32) * scale) as u8,
    (blue_a as f32 + (blue_b as f32 - blue_a as f32) * scale) as u8,
    (alpha_a as f32 + (alpha_b as f32 - alpha_a as f32) * scale) as u8,
  )
}
