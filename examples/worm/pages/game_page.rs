use flut::widgets::{
  stateful_widget::State, widget::*, Column, Grid, RectWidget, StatefulWidget, Widget,
};
use skia_safe::{Color, Rect};

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GamePage;

impl<'a> StatefulWidget<'a> for GamePage {
  fn new_state(&self) -> Box<dyn State<'a> + 'a> {
    Box::new(GamePageState {})
  }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct GamePageState {}

impl<'a> State<'a> for GamePageState {
  fn build(&self, _constraint: Rect) -> Widget<'a> {
    Column {
      children: vec![Grid {
        col_count: 41,
        row_count: 41,
        gap: 2.0,
        builder: Box::new(|_| {
          RectWidget {
            color: Color::DARK_GRAY,
          }
          .into_widget()
        }),
      }
      .into_widget()],
    }
    .into_widget()
  }
}
