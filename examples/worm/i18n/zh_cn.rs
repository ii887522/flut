use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("worm", "贪食虫"),
        ("start_game", "开始游戏"),
        ("exit_game", "退出游戏"),
        ("you_died", "你死了..."),
        ("give_up", "放弃"),
        ("try_again", "再试一次"),
        ("you_died_desc", "您吃了{score}个青苹果。想再试一次吗？"),
      ]),
    );
}
