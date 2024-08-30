use rand::{distributions::Standard, prelude::*};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Direction {
  Up,
  Right,
  Down,
  Left,
}

impl Distribution<Direction> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
    match rng.gen_range(0..4) {
      0 => Direction::Up,
      1 => Direction::Right,
      2 => Direction::Down,
      _ => Direction::Left,
    }
  }
}
