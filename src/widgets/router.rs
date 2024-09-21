use super::{stateful_widget::State, StatefulWidget, Widget};
use crate::helpers::AnimationCount;
use skia_safe::Rect;
use std::{
  collections::HashMap,
  fmt::Debug,
  mem,
  sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Router<'a> {
  navigator: Arc<Mutex<Navigator>>,
  children: HashMap<String, Widget<'a>>,
}

impl<'a> Router<'a> {
  pub fn new(
    initial_route: String,
    children: impl Fn(Arc<Mutex<Navigator>>) -> HashMap<String, Widget<'a>> + 'a + Send,
  ) -> Self {
    let navigator = Arc::new(Mutex::new(Navigator::new(initial_route)));

    Self {
      children: children(Arc::clone(&navigator)),
      navigator,
    }
  }
}

impl<'a> StatefulWidget<'a> for Router<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(RouterState {
      navigator: mem::take(&mut self.navigator),
      children: mem::take(&mut self.children),
    })
  }
}

#[derive(Debug)]
struct RouterState<'a> {
  navigator: Arc<Mutex<Navigator>>,
  children: HashMap<String, Widget<'a>>,
}

impl<'a> State<'a> for RouterState<'a> {
  fn update(&mut self, _dt: f32) -> bool {
    let mut navigator = self.navigator.lock().unwrap();

    if *navigator.animation_count == 0 {
      false
    } else {
      navigator.animation_count = AnimationCount::new();
      true
    }
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let navigator = self.navigator.lock().unwrap();
    Widget::clone(&self.children[&navigator.current_route])
  }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Navigator {
  current_route: String,
  animation_count: AnimationCount,
}

impl Navigator {
  const fn new(initial_route: String) -> Self {
    Self {
      current_route: initial_route,
      animation_count: AnimationCount::new(),
    }
  }

  pub fn go(&mut self, route: String) {
    self.current_route = route;

    // To wake up router state to trigger rebuild
    self.animation_count.incr();
  }
}
