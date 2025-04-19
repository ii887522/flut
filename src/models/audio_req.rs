pub enum AudioReq {
  LoadMusic {
    file_path: &'static str,
  },
  LoadSound {
    file_path: &'static str,
  },
  PlaySound {
    file_path: &'static str,
  },
  PlayMusic {
    file_path: &'static str,
    volume: i32,
  },
  StopMusic,
}
