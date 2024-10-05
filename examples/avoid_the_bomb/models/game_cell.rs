#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GameCell {
  Count(u8),
  Bomb,
  Flag,
}

impl Default for GameCell {
  fn default() -> Self {
    Self::Count(0)
  }
}
