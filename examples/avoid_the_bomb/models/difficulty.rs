#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Difficulty {
  Easy,
  #[default]
  Medium,
  Hard,
}

impl Difficulty {
  pub(crate) const fn get_bomb_count(&self) -> usize {
    match self {
      Difficulty::Easy => 50,
      Difficulty::Medium => 100,
      Difficulty::Hard => 200,
    }
  }
}

impl From<&str> for Difficulty {
  fn from(difficulty: &str) -> Self {
    match difficulty.to_lowercase().as_str() {
      "easy" => Difficulty::Easy,
      "medium" => Difficulty::Medium,
      "hard" => Difficulty::Hard,
      _ => Difficulty::Medium,
    }
  }
}

impl From<String> for Difficulty {
  fn from(difficulty: String) -> Self {
    difficulty.as_str().into()
  }
}

impl From<&String> for Difficulty {
  fn from(difficulty: &String) -> Self {
    difficulty.as_str().into()
  }
}
