use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("avoid_the_bomb", "远离炸弹"),
        ("start_easy_game", "开始简单游戏"),
        ("start_medium_game", "开始普通游戏"),
        ("start_hard_game", "开始困难游戏"),
        ("exit_game", "退出游戏"),
        ("you_died", "你死了..."),
        ("give_up", "放弃"),
        ("try_again", "再试一次"),
        ("you_died_desc", "你被炸弹炸死了。想再试一次吗？"),
        ("you_won", "你赢了~~"),
        ("home", "主页"),
        ("you_won_desc", "您标记了所有炸弹位置。想再试一次吗？"),
        ("back_home", "返回主页?"),
        ("back_home_desc", "您确定要放弃这个游戏吗？你的进度将会丢失！"),
        ("close", "关闭"),
      ]),
    );
}
