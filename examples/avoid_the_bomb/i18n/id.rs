use std::{cell::RefCell, collections::HashMap};

thread_local! {
  pub(super) static MESSAGES: RefCell<HashMap<&'static str, &'static str>> =
    RefCell::new(
      HashMap::from_iter([
        ("avoid_the_bomb", "Avoid The Bomb"),
      ]),
    );
}
