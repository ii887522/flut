use crate::models::AudioTask;
use sdl2::mixer::{self, Channel, Chunk};
use std::{collections::HashMap, sync::mpsc::Receiver};

pub(crate) fn main(rx: Receiver<AudioTask<'_>>) {
  if mixer::open_audio(48000, mixer::AUDIO_F32SYS, 2, 2048).is_err() {
    return;
  }

  let mut chunk_map = HashMap::new();

  while let Ok(task) = rx.recv() {
    match task {
      AudioTask::LoadSound(file_path) => {
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
