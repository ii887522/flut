use std::cmp::Ordering;
use voracious_radix_sort::Radixable;

#[derive(Clone, Copy, Debug, Eq)]
pub struct Write {
  pub index: u32,
  pub len: u32,
}

impl PartialEq for Write {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.index == other.index
  }
}

impl PartialOrd for Write {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Write {
  #[inline]
  fn cmp(&self, other: &Self) -> Ordering {
    self.index.cmp(&other.index)
  }
}

impl Radixable<u32> for Write {
  type Key = u32;

  #[inline]
  fn key(&self) -> Self::Key {
    self.index
  }
}
