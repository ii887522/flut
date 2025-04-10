#[derive(Clone, Copy)]
pub(crate) enum Direction {
  Up,
  Right,
  Down,
  Left,
}

impl Direction {
  pub(crate) fn rand() -> Self {
    match fastrand::u8(0..4) {
      0 => Self::Up,
      1 => Self::Right,
      2 => Self::Down,
      _ => Self::Left,
    }
  }
}
