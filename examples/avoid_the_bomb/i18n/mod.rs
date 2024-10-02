use flut::{helpers::I18n, models::Lang};
use std::collections::HashMap;

mod en;
mod id;
mod zh_cn;
mod zh_tw;

thread_local! {
  pub(super) static I18N: I18n = I18n::new(HashMap::from_iter([
    (Lang::En, en::MESSAGES.take()),
    (Lang::Id, id::MESSAGES.take()),
    (Lang::ZhCn, zh_cn::MESSAGES.take()),
    (Lang::ZhTw, zh_tw::MESSAGES.take()),
  ]))
  .call();
}
