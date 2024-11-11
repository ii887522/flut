use super::Direction;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct WormCell {
  pub(crate) position: u16,
  pub(crate) direction: Direction,
}
