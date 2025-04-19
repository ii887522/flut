use crate::models::AudioReq;
use sdl2::mixer::{self, Channel, Chunk, InitFlag, Music};
use std::{collections::HashMap, sync::mpsc::Receiver};

pub(super) fn run(rx: Receiver<AudioReq>) {
  let mixer = mixer::init(InitFlag::MP3);

  if mixer.is_err() {
    return;
  }

  if mixer::open_audio(44100, mixer::AUDIO_S16SYS, 2, 1024).is_err() {
    return;
  }

  let mut path_to_chunk = HashMap::new();
  let mut path_to_music = HashMap::new();

  for req in rx.iter() {
    match req {
      AudioReq::LoadMusic { file_path } => {
        if let Ok(music) = Music::from_file(file_path) {
          path_to_music.insert(file_path, music);
        }
      }
      AudioReq::LoadSound { file_path } => {
        if let Ok(chunk) = Chunk::from_file(file_path) {
          path_to_chunk.insert(file_path, chunk);
        }
      }
      AudioReq::PlaySound { file_path } => {
        if let Some(chunk) = path_to_chunk.get(file_path) {
          let _ = Channel::all().play(chunk, 0);
        }
      }
      AudioReq::PlayMusic { file_path, volume } => {
        if let Some(music) = path_to_music.get(file_path) {
          Music::set_volume(volume);
          let _ = music.play(-1);
        }
      }
      AudioReq::StopMusic => Music::halt(),
    }
  }
}
