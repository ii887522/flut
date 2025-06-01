#[derive(Clone, Copy, Default, Debug)]
pub enum Anchor {
  #[default]
  TopLeft,
  Top,
  TopRight,
  Left,
  Center,
  Right,
  BottomLeft,
  Bottom,
  BottomRight,
}
