use flut::{helpers::I18n, models::Locale};
use std::collections::HashMap;

mod en;
mod id;
mod zh_cn;
mod zh_tw;

thread_local! {
  pub(super) static I18N: I18n = I18n::new(HashMap::from_iter([
    (Locale::En, en::MESSAGES.take()),
    (Locale::Id, id::MESSAGES.take()),
    (Locale::ZhCn, zh_cn::MESSAGES.take()),
    (Locale::ZhTw, zh_tw::MESSAGES.take()),
  ]))
  .call();
}
