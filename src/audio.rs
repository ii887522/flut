use crate::models::audio_req::AudioReq;
use awedio::{Sound as _, sounds};
use rustc_hash::FxHashMap;
use std::sync::mpsc::Receiver;

pub fn main(rx: Receiver<AudioReq>) {
  let (mut audio_manager, _audio_backend) = match awedio::start() {
    Ok((audio_manager, audio_backend)) => (audio_manager, audio_backend),
    Err(err) => {
      eprintln!("Failed to start outputting audio: {err}");
      return;
    }
  };

  let mut path_to_sound = FxHashMap::default();

  for req in rx {
    match req {
      AudioReq::PlaySound(sound_path) => {
        if !path_to_sound.contains_key(sound_path) {
          let sound = match sounds::open_file(sound_path) {
            Ok(sound) => sound,
            Err(err) => {
              eprintln!("Failed to open {sound_path}: {err}");
              continue;
            }
          };

          let sound = match sound.into_memory_sound() {
            Ok(sound) => sound,
            Err(err) => {
              eprintln!("Failed to load {sound_path}: {err}");
              continue;
            }
          };

          path_to_sound.insert(sound_path, sound);
        }

        audio_manager.play(Box::new(path_to_sound[sound_path].clone()));
      }
    }
  }
}
