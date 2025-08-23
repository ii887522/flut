use crate::{Renderer, models::AudioReq};
use std::sync::mpsc::Sender;

pub struct Context<'renderer> {
  pub renderer: &'renderer mut dyn Renderer,
  pub audio_tx: Sender<AudioReq>,
  pub app_size: (u32, u32),
}
