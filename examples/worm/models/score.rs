use crate::consts;
use flut::models::{Anchor, Text};

#[derive(Clone, Copy)]
pub(crate) struct Score {
  pub(crate) score: u32,
  pub(crate) drawable_id: u16,
}

impl From<Score> for Text {
  fn from(score: Score) -> Self {
    Text::new(
      consts::SCORE_POSITION,
      (255, 255, 255, 255),
      score.score.to_string().into(),
    )
    .anchor(Anchor::Top)
    .call()
  }
}
