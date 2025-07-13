use voracious_radix_sort::Radixable;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(super) struct Write {
  from: u32,
  size: u32,
}

impl Write {
  pub(super) const fn new(from: u32, size: u32) -> Self {
    Self { from, size }
  }
}

impl Radixable<u32> for Write {
  type Key = u32;

  fn key(&self) -> Self::Key {
    self.from
  }
}

// todo: WIP
pub(super) fn coalesce(writes: &[Write]) {
  if writes.is_empty() {
    return;
  }

  let mut sorted_writes = writes.to_vec();
  sorted_writes.sort_unstable_by_key(|w| w.from);

  let mut coalesced = Vec::with_capacity(sorted_writes.len());
  let mut current = sorted_writes[0];

  for &write in &sorted_writes[1..] {
    if write.from == current.from + current.size {
      current.size += write.size;
    } else {
      coalesced.push(current);
      current = write;
    }
  }
  coalesced.push(current);

  sorted_writes.clear();
  sorted_writes.extend(coalesced);
}
