use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("worm", "Cacing"),
        ("start_game", "Mulai Permainan"),
        ("exit_game", "Keluar dari Permainan"),
        ("you_died", "Kamu Meninggal..."),
        ("give_up", "Menyerah"),
        ("try_again", "Coba Lagi"),
        ("you_died_desc", "Anda makan {score} hijau apel. Ingin mencoba lagi?"),
      ]),
    );
}
