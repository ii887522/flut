pub mod audio_manager;
pub mod clock;
pub mod sdf;

pub use audio_manager::AudioManager;
pub use clock::Clock;

use crate::models::Write;
use voracious_radix_sort::RadixSort;

pub fn coalesce_writes(writes: &mut Vec<Write>) {
  if writes.is_empty() {
    return;
  }

  writes.voracious_mt_sort(0);

  let mut results = Vec::with_capacity(writes.len());
  results.push(writes[0]);

  for &write in &writes[1..] {
    let result = results.last_mut().unwrap();

    if write.index <= result.index + result.len {
      result.len = result.len.max(write.index + write.len - result.index);
    } else {
      results.push(write);
    }
  }

  *writes = results;
}

#[inline]
pub const fn pack_color(color: (u8, u8, u8, u8)) -> u32 {
  (color.0 as u32) << 24 | (color.1 as u32) << 16 | (color.2 as u32) << 8 | (color.3 as u32)
}
