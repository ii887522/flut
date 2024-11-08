use flut::widgets::{widget::*, BuilderWidget, Grid, RectWidget, Widget};
use skia_safe::{Color, Rect};

#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GamePage;

impl<'a> BuilderWidget<'a> for GamePage {
  fn build(&self, _constraint: Rect) -> Widget<'a> {
    Grid {
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
    .into_widget()
  }
}
