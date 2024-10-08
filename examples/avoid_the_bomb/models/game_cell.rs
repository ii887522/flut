use super::GameCellState;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GameCell {
  Count { count: u8, state: GameCellState },
  Bomb { state: GameCellState },
}

impl Default for GameCell {
  fn default() -> Self {
    Self::Count {
      count: 0,
      state: GameCellState::Hidden,
    }
  }
}
