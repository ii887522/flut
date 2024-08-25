#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GameCell {
  #[default]
  Air,
  Worm,
  Wall,
  Food,
}
