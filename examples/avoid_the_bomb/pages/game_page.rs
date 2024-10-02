use flut::widgets::{
  router::Navigator, stateful_widget::State, widget::*, Button, Column, Grid, StatefulWidget,
  Widget,
};
use skia_safe::{Color, Rect};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub(crate) struct GamePage<'a> {
  pub(crate) navigator: Arc<Mutex<Navigator<'a>>>,
}

impl<'a> StatefulWidget<'a> for GamePage<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    Box::new(GamePageState)
  }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct GamePageState;

impl<'a> State<'a> for GamePageState {
  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    Column::new()
      .children(vec![Grid {
        col_count: 31,
        row_count: 31,
        gap: 2.0,
        builder: Box::new(|index| {
          Button {
            bg_color: Color::from_rgb(56, 56, 56),
            border_radius: 0.0,
            is_elevated: false,
            is_cursor_fixed: true,
            has_effect: false,
            on_mouse_up: Arc::new(Mutex::new(move || {
              // todo: Do something
              dbg!(index);
            })),
            ..Default::default()
          }
          .into_widget()
        }),
      }
      .into_widget()])
      .call()
      .into_widget()
  }
}
