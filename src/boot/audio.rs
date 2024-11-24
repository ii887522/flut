use crate::models::AudioReq;
use sdl2::mixer::{self, Channel, Chunk};
use std::{collections::HashMap, sync::mpsc::Receiver};

pub(crate) fn main(rx: Receiver<AudioReq>) {
  let _ = mixer::open_audio(48000, mixer::AUDIO_F32SYS, 2, 2048);
  let mut chunk_map = HashMap::new();

  for req in rx {
    match req {
      AudioReq::PlaySound(file_path) => {
        if let Ok(chunk) = chunk_map
          .entry(file_path)
          .or_insert_with(|| Chunk::from_file(file_path))
        {
          let _ = Channel::all().play(chunk, 0);
        }
      }
    }
  }
}
