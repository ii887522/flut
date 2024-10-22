#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioReq<'a> {
  LoadSound(&'a str),
  PlaySound(&'a str),
}
