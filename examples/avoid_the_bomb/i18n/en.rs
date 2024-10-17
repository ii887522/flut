use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("avoid_the_bomb", "Avoid The Bomb"),
        ("start_easy_game", "Start Easy Game"),
        ("start_medium_game", "Start Medium Game"),
        ("start_hard_game", "Start HARD Game"),
        ("exit_game", "Exit Game"),
        ("you_died", "You Died..."),
        ("give_up", "Give Up"),
        ("try_again", "Try Again"),
        ("you_died_desc", "You killed by the bomb. Want to try again?"),
        ("you_won", "You Won~~"),
        ("home", "Home"),
        ("you_won_desc", "You marked all the bomb locations. Want try one more time?"),
        ("back_home", "Back to home?"),
        ("back_home_desc", "Are you sure you want to give up this game? Your progress will be LOST!"),
        ("close", "Close"),
      ]),
    );
}
