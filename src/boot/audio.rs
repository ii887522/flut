use crate::models::AudioTask;
use sdl2::mixer::{self, Channel, Chunk};
use std::{collections::HashMap, sync::mpsc::Receiver};

pub(crate) fn main(rx: Receiver<AudioTask<'_>>) {
  let _ = mixer::open_audio(48000, mixer::AUDIO_F32SYS, 2, 2048);
  let mut chunk_map = HashMap::new();

  for task in rx {
    match task {
      AudioTask::LoadSound(file_path) => {
        if chunk_map.contains_key(file_path) {
          continue;
        }

        if let Ok(chunk) = Chunk::from_file(file_path) {
          chunk_map.insert(file_path, chunk);
        }
      }
      AudioTask::PlaySound(file_path) => {
        if let Some(chunk) = chunk_map.get(file_path) {
          let _ = Channel::all().play(chunk, 0);
        }
      }
    }
  }
}
