#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum HorizontalAlign {
  #[default]
  Left,
  Center,
  Right,
}
