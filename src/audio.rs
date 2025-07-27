use crate::models::AudioReq;
use flut_macro::warn;
use rustc_hash::FxHashMap;
use sdl2::mixer::{self, Channel, Chunk, Music};
use std::sync::mpsc::Receiver;

pub(super) fn run(rx: Receiver<AudioReq>) {
  let _ = match mixer::init(mixer::InitFlag::MP3) {
    Ok(mixer) => mixer,
    Err(err) => {
      warn!("{err}");
      return;
    }
  };

  match mixer::open_audio(44100, mixer::AUDIO_F32SYS, 2, 2048) {
    Ok(_) => {}
    Err(err) => {
      warn!("{err}");
      return;
    }
  }

  let mut path_to_music = FxHashMap::default();
  let mut path_to_sound = FxHashMap::default();

  for req in rx.iter() {
    match req {
      AudioReq::HaltMusic => Music::halt(),
      AudioReq::PlayMusic(path) => {
        let music = if let Some(music) = path_to_music.get(&path) {
          music
        } else {
          let music = match Music::from_file(&*path) {
            Ok(music) => music,
            Err(err) => {
              warn!("{err}");
              continue;
            }
          };

          path_to_music.insert(path.clone(), music);
          &path_to_music[&path]
        };

        Music::set_volume(32);

        match music.play(-1) {
          Ok(_) => {}
          Err(err) => warn!("{err}"),
        }
      }
      AudioReq::PlaySound(path) => {
        let sound = if let Some(sound) = path_to_sound.get(&path) {
          sound
        } else {
          let sound = match Chunk::from_file(&*path) {
            Ok(sound) => sound,
            Err(err) => {
              warn!("{err}");
              continue;
            }
          };

          path_to_sound.insert(path.clone(), sound);
          &path_to_sound[&path]
        };

        match Channel::all().play(sound, 0) {
          Ok(_) => {}
          Err(err) => warn!("{err}"),
        }
      }
    }
  }

  mixer::close_audio();
}
