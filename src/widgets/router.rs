use super::{stateful_widget::State, StatefulWidget, Widget};
use crate::{helpers::AnimationCount, models::Route};
use skia_safe::Rect;
use std::{
  collections::HashMap,
  fmt::{self, Debug, Formatter},
  mem,
  sync::{Arc, Mutex},
};

type QsParams<'a> = HashMap<&'a str, &'a str>;
type ChildBuilder<'a> = dyn Fn(QsParams<'_>) -> Widget<'a> + 'a + Send + Sync;

pub struct Router<'a> {
  navigator: Arc<Mutex<Navigator<'a>>>,
  children: HashMap<&'a str, Box<ChildBuilder<'a>>>,
}

impl Debug for Router<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("Router")
      .field("navigator", &self.navigator)
      .finish_non_exhaustive()
  }
}

impl<'a> Router<'a> {
  pub fn new(
    initial_route: &'a str,
    children: impl Fn(
        Arc<Mutex<Navigator<'a>>>,
      ) -> HashMap<&str, Box<dyn Fn(QsParams<'_>) -> Widget<'a> + 'a + Send + Sync>>
      + 'a,
  ) -> Self {
    let navigator = Arc::new(Mutex::new(Navigator::new(initial_route)));
    let children = children(Arc::clone(&navigator));

    Self {
      navigator,
      children,
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

struct RouterState<'a> {
  navigator: Arc<Mutex<Navigator<'a>>>,
  children: HashMap<&'a str, Box<ChildBuilder<'a>>>,
}

impl Debug for RouterState<'_> {
  fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
    fmt
      .debug_struct("RouterState")
      .field("navigator", &self.navigator)
      .finish_non_exhaustive()
  }
}

impl<'a> State<'a> for RouterState<'a> {
  fn update(&mut self, _dt: f32) -> bool {
    let mut navigator = self.navigator.lock().unwrap();

    if *navigator.animation_count == 0 {
      return false;
    }

    navigator.animation_count = AnimationCount::new();
    true
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let navigator = self.navigator.lock().unwrap();
    let route = Route::from_relative_url(navigator.current_route);
    self.children[route.path](route.qs_params)
  }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Navigator<'a> {
  current_route: &'a str,
  animation_count: AnimationCount,
}

impl<'a> Navigator<'a> {
  const fn new(initial_route: &'a str) -> Self {
    Self {
      current_route: initial_route,
      animation_count: AnimationCount::new(),
    }
  }

  pub fn go(&mut self, route: &'a str) {
    self.current_route = route;

    // To wake up router state to trigger rebuild
    self.animation_count.incr();
  }
}
