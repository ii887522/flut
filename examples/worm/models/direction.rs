#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Direction {
  Up,
  Right,
  Down,
  Left,
}

impl Direction {
  const ALL: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
  ];

  pub(crate) fn rand() -> Self {
    fastrand::choice(Self::ALL).unwrap()
  }
}
