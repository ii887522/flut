use std::borrow::Cow;

pub(crate) struct TextId {
  pub(crate) glyph_ids: Box<[u32]>,
  pub(crate) text: Cow<'static, str>,
}
