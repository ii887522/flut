#[derive(Clone, Copy, Default)]
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
