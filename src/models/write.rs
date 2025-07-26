use voracious_radix_sort::Radixable;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct Write {
  pub(crate) from: u32,
  pub(crate) size: u32,
}

impl Radixable<u32> for Write {
  type Key = u32;

  fn key(&self) -> Self::Key {
    self.from
  }
}
