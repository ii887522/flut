#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerticalAlign {
  #[default]
  Top,
  Middle,
  Bottom,
}
