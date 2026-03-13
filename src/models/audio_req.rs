#[derive(Clone, Copy)]
pub enum AudioReq {
  PlaySound(&'static str),
}
