#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod animations;
pub mod app;
mod audio;
pub mod clock;
pub mod collections;
mod consts;
pub mod context;
mod id_generator;
pub mod models;
pub mod renderer;
mod vk;
pub mod widgets;

use crate::models::Write;
pub use app::App;
pub use clock::Clock;
pub use context::Context;
use id_generator::IdGenerator;
use rayon::prelude::*;
pub use renderer::Renderer;
use std::ffi::c_void;
use std::{collections::VecDeque, ptr};
use std::{mem, slice};
use voracious_radix_sort::RadixSort;

const fn as_bytes<T>(item: &T) -> &[u8] {
  unsafe { slice::from_raw_parts(item as *const _ as *const _, mem::size_of::<T>()) }
}

const fn pack_color(color: (u8, u8, u8, u8)) -> u32 {
  ((color.0 as u32) << 24) | ((color.1 as u32) << 16) | ((color.2 as u32) << 8) | (color.3 as u32)
}

const fn unpack_color(color: u32) -> (u8, u8, u8, u8) {
  (
    ((color >> 24) & 0xFF) as u8,
    ((color >> 16) & 0xFF) as u8,
    ((color >> 8) & 0xFF) as u8,
    (color & 0xFF) as u8,
  )
}

fn coalesce(writes: &mut Vec<Write>) {
  writes.voracious_mt_sort(0);

  let results = writes
    .par_iter()
    .fold(Vec::new, |mut writes: Vec<Write>, &write| {
      if let Some(last_write) = writes.last_mut() {
        if write.from <= last_write.from + last_write.size {
          last_write.size = last_write
            .size
            .max(write.from + write.size - last_write.from);
        } else {
          writes.push(write);
        }
      } else {
        writes.push(write);
      }

      writes
    })
    .reduce(Vec::new, |mut writes_a, writes_b| {
      if let Some((last_write_a, first_write_b)) = writes_a.last_mut().zip(writes_b.first()) {
        if first_write_b.from <= last_write_a.from + last_write_a.size {
          last_write_a.size = last_write_a
            .size
            .max(first_write_b.from + first_write_b.size - last_write_a.from);

          writes_a.par_extend(writes_b.into_par_iter().skip(1));
        } else {
          writes_a.par_extend(writes_b);
        }
      } else {
        writes_a.par_extend(writes_b);
      }

      writes_a
    });

  *writes = results;
}

fn flush_writes<T>(writes_queues: &mut VecDeque<Vec<Write>>, src: *const T, dst: *mut c_void) {
  let writes = writes_queues.back_mut().unwrap();
  coalesce(writes);

  unsafe {
    for writes in &mut *writes_queues {
      for write in writes {
        ptr::copy_nonoverlapping(
          src.add(write.from as _),
          (dst as *mut T).add(write.from as _),
          write.size as _,
        );
      }
    }
  }

  if writes_queues.len() >= crate::consts::MAX_IN_FLIGHT_FRAME_COUNT {
    writes_queues.pop_front().unwrap();
  }

  writes_queues.push_back(vec![]);
}
