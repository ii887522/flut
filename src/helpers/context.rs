use crate::models::AudioTask;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Context<'a> {
  pub audio_tx: Option<Sender<AudioTask<'a>>>,
}
