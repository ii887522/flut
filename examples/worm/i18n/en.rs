use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("worm", "Worm"),
        ("start_game", "Start Game"),
        ("exit_game", "Exit Game"),
        ("you_died", "You Died..."),
        ("give_up", "Give Up"),
        ("try_again", "Try Again"),
        ("you_died_desc", "You ate {score} green {score:apple|apples}. Want to try again?"),
      ]),
    );
}
