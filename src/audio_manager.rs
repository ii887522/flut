use kira::sound::{
  FromFileError,
  static_sound::{StaticSoundData, StaticSoundHandle},
  streaming::{StreamingSoundData, StreamingSoundHandle},
};
use rustc_hash::FxHashMap;

pub struct AudioManager {
  audio_manager: Option<kira::AudioManager>,
  file_path_to_sound: FxHashMap<&'static str, Result<StaticSoundData, FromFileError>>,
}

impl AudioManager {
  pub(super) fn new() -> Self {
    Self {
      audio_manager: match kira::AudioManager::<kira::DefaultBackend>::new(
        kira::AudioManagerSettings::default(),
      ) {
        Ok(audio_manager) => Some(audio_manager),
        Err(err) => {
          println!("Failed to create audio manager: {err}");
          None
        }
      },
      file_path_to_sound: FxHashMap::default(),
    }
  }

  pub fn play_sound(&mut self, sound_file_path: &'static str) -> Option<StaticSoundHandle> {
    let Some(audio_manager) = &mut self.audio_manager else {
      return None;
    };

    let sound = match self
      .file_path_to_sound
      .entry(sound_file_path)
      .or_insert_with(|| StaticSoundData::from_file(sound_file_path))
    {
      Ok(sound) => sound,
      Err(err) => {
        println!("Failed to load sound: {err}");
        return None;
      }
    };

    match audio_manager.play(sound.clone()) {
      Ok(sound) => Some(sound),
      Err(err) => {
        println!("Failed to play sound: {err}");
        None
      }
    }
  }

  pub fn play_music(
    &mut self,
    music_file_path: &'static str,
  ) -> Option<StreamingSoundHandle<FromFileError>> {
    let Some(audio_manager) = &mut self.audio_manager else {
      return None;
    };

    let music = match StreamingSoundData::from_file(music_file_path) {
      Ok(music) => music,
      Err(err) => {
        println!("Failed to load music: {err}");
        return None;
      }
    };

    match audio_manager.play(music) {
      Ok(music) => Some(music),
      Err(err) => {
        println!("Failed to play music: {err}");
        None
      }
    }
  }
}
