#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod app;
mod audio;
mod batches;
mod buffers;
pub mod clock;
pub mod collections;
mod engine;
mod font_atlas;
mod images;
pub mod models;
mod pipelines;
mod shaders;

pub use app::App;
pub use app::AppConfig;
pub use clock::Clock;
pub use engine::Engine;

use rayon::prelude::*;
use std::{
  mem,
  ops::{Bound, RangeBounds},
  ptr,
};

const unsafe fn as_bytes<T>(from: &T) -> &[u8] {
  unsafe { &*ptr::slice_from_raw_parts(from as *const _ as *const _, mem::size_of::<T>()) }
}

pub fn par_swap_remove<T: Send>(vec: &mut Vec<T>, indices: impl RangeBounds<usize>) {
  let start_index = match indices.start_bound() {
    Bound::Included(&start_index) => start_index,
    Bound::Excluded(&start_index) => start_index + 1,
    Bound::Unbounded => 0,
  };

  let end_index = match indices.end_bound() {
    Bound::Included(&end_index) => end_index,
    Bound::Excluded(&end_index) => end_index - 1,
    Bound::Unbounded => vec.len() - 1,
  };

  let index_count = end_index - start_index + 1;
  let vec_start_index = start_index.max(vec.len() - index_count);

  if start_index == vec_start_index {
    // If remove last elements from vec, can simply remove
    vec.truncate(start_index);
  } else if end_index + 1 >= vec_start_index {
    // If remove elements from vec leaving few elements behind, Vec::drain() is sufficiently fast
    vec.par_drain(indices);
  } else {
    // Perform operation similar to Vec::swap_remove() but for a range of elements removal
    unsafe {
      ptr::copy_nonoverlapping(
        vec[vec_start_index..].as_ptr(),
        vec[start_index..=end_index].as_mut_ptr(),
        index_count,
      );
    }

    vec.truncate(vec_start_index);
  }
}

const fn pack_color(color: (u8, u8, u8)) -> u32 {
  ((color.0 as u32) << 24) | ((color.1 as u32) << 16) | ((color.2 as u32) << 8) | 0xFF
}

const fn unpack_color(color: u32) -> (u8, u8, u8) {
  ((color >> 24) as u8, (color >> 16) as u8, (color >> 8) as u8)
}
