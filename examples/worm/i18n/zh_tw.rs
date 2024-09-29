use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("worm", "貪食蟲"),
        ("start_game", "開始遊戲"),
        ("exit_game", "退出遊戲"),
        ("you_died", "你死了..."),
        ("give_up", "放棄"),
        ("try_again", "再試一次"),
        ("you_died_desc", "您吃了{score}個青蘋果。想再試一次嗎？"),
      ]),
    );
}
