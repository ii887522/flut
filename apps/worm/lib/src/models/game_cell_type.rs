#[derive(Clone, Copy)]
pub(crate) enum GameCellType {
  Air,
  Wall,
  Worm,
  Food,
}

impl GameCellType {
  #[inline]
  pub(crate) const fn get_color(&self) -> (f32, f32, f32, f32) {
    match self {
      GameCellType::Air => (0.15, 0.15, 0.15, 1.0),
      GameCellType::Wall => (1.0, 0.0, 0.0, 1.0),
      GameCellType::Worm => (0.918, 0.663, 0.596, 1.0),
      GameCellType::Food => (0.0, 1.0, 0.0, 1.0),
    }
  }
}
