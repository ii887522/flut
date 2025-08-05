use crate::consts;
use flut::models::{Anchor, Text};

#[derive(Clone, Copy)]
pub(crate) struct Score {
  pub(crate) score: u32,
  pub(crate) drawable_id: u32,
}

impl From<Score> for Text {
  fn from(score: Score) -> Self {
    Self::new()
      .position(((consts::APP_SIZE.0 >> 1) as _, consts::SCORE_MARGIN_TOP))
      .font_size(48u16)
      .text(score.score.to_string())
      .color((255, 255, 255, 255))
      .anchor(Anchor::Top)
      .call()
  }
}
