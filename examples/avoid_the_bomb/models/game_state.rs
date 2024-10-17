#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameState {
  #[default]
  Playing,
  Pause,
  Dead,
  Won,
}
