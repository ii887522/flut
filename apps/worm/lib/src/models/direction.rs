#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
  Up,
  Right,
  Down,
  Left,
}

impl Direction {
  #[inline]
  pub(crate) fn rand() -> Self {
    match fastrand::u32(..4) {
      0 => Self::Up,
      1 => Self::Right,
      2 => Self::Down,
      3 => Self::Left,
      _ => unreachable!(),
    }
  }
}
