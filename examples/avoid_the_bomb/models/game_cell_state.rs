#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GameCellState {
  #[default]
  Hidden,
  Visible,
  Flagged,
}
