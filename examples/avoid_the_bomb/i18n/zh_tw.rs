use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("avoid_the_bomb", "遠離炸彈"),
        ("start_easy_game", "開始簡單遊戲"),
        ("start_medium_game", "開始普通遊戲"),
        ("start_hard_game", "開始困難遊戲"),
        ("exit_game", "退出遊戲"),
        ("you_died", "你死了..."),
        ("give_up", "放棄"),
        ("try_again", "再試一次"),
        ("you_died_desc", "你被炸彈炸死了。想再試一次嗎？"),
        ("you_won", "你贏了~~"),
        ("home", "首頁"),
        ("you_won_desc", "您標記了所有炸彈位置。想再試一次嗎？"),
        ("back_home", "返回首頁?"),
        ("back_home_desc", "您確定要放棄這個遊戲嗎？你的進度將會失去！"),
        ("close", "關閉"),
      ]),
    );
}
