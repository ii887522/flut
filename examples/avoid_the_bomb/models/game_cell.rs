#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GameCell {
  Count { count: u8, is_visible: bool },
  Bomb { is_visible: bool },
  Flag,
}

impl Default for GameCell {
  fn default() -> Self {
    Self::Count {
      count: 0,
      is_visible: false,
    }
  }
}
