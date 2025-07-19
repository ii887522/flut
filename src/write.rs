use rayon::prelude::*;
use voracious_radix_sort::{RadixSort, Radixable};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(super) struct Write {
  from: u32,
  size: u32,
}

impl Write {
  pub(super) const fn new(from: u32, size: u32) -> Self {
    Self { from, size }
  }

  pub(super) const fn get_from(&self) -> u32 {
    self.from
  }

  pub(super) const fn get_size(&self) -> u32 {
    self.size
  }
}

impl Radixable<u32> for Write {
  type Key = u32;

  fn key(&self) -> Self::Key {
    self.from
  }
}

pub(super) fn coalesce(writes: &mut Vec<Write>) {
  writes.voracious_mt_sort(0);

  let results = writes
    .par_iter()
    .fold(Vec::new, |mut writes: Vec<Write>, &write| {
      if let Some(last_write) = writes.last_mut() {
        if write.from <= last_write.from + last_write.size {
          last_write.size = write.from + write.size - last_write.from
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
          last_write_a.size = first_write_b.from + first_write_b.size - last_write_a.from;
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
