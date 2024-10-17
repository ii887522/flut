use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("avoid_the_bomb", "Hindari Bom"),
        ("start_easy_game", "Mulai Permainan Sederhana"),
        ("start_medium_game", "Mulai Permainan Normal"),
        ("start_hard_game", "Mulai Permainan SULIT"),
        ("exit_game", "Keluar dari Permainan"),
        ("you_died", "Kamu Meninggal..."),
        ("give_up", "Menyerah"),
        ("try_again", "Coba Lagi"),
        ("you_died_desc", "Anda terbunuh oleh bom. Ingin mencoba lagi?"),
        ("you_won", "Anda Menang~~"),
        ("home", "Rumah"),
        ("you_won_desc", "Anda menandai semua lokasi bom. Coba sekali lagi?"),
        ("back_home", "Kembali ke Rumah?"),
        ("back_home_desc", "Apakah anda yakin ingin menghentikan permainan ini? Kemajuan anda akan HILANG!"),
        ("close", "Menutup"),
      ]),
    );
}
