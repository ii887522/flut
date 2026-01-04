#[derive(Clone, Copy)]
pub(crate) enum GameCellType {
  Air,
  Wall,
  Worm,
  Food,
}

impl GameCellType {
  #[inline]
  pub(crate) const fn get_color(&self) -> (u8, u8, u8, u8) {
    match self {
      GameCellType::Air => (38, 38, 38, 255),
      GameCellType::Wall => (255, 0, 0, 255),
      GameCellType::Worm => (235, 170, 153, 255),
      GameCellType::Food => (0, 255, 0, 255),
    }
  }
}
