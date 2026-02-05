use std::cmp::Ordering;
use voracious_radix_sort::Radixable;

#[derive(Clone, Copy)]
pub struct Range {
  pub start: u32,
  pub end: u32,
}

impl PartialEq for Range {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.start == other.start
  }
}

impl PartialOrd for Range {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.start.partial_cmp(&other.start)
  }
}

impl Radixable<u32> for Range {
  type Key = u32;

  #[inline]
  fn key(&self) -> Self::Key {
    self.start
  }
}
