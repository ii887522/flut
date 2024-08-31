#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioTask<'a> {
  LoadSound(&'a str),
  PlaySound(&'a str),
}
