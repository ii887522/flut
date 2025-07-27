use std::borrow::Cow;

pub enum AudioReq {
  HaltMusic,
  PlayMusic(Cow<'static, str>),
  PlaySound(Cow<'static, str>),
}
