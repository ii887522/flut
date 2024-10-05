use crate::models::GameCell;
use flut::widgets::{
  router::Navigator, stateful_widget::State, widget::*, Button, Column, Grid, ImageWidget,
  StatefulWidget, Widget,
};
use rand::{prelude::*, seq::index};
use rayon::prelude::*;
use skia_safe::{Color, Rect};
use std::sync::{Arc, Mutex, RwLock};

const COL_COUNT: u16 = 31;
const ROW_COUNT: u16 = 31;

#[derive(Debug, Default)]
pub(crate) struct GamePage<'a> {
  pub(crate) navigator: Arc<Mutex<Navigator<'a>>>,
}

impl<'a> StatefulWidget<'a> for GamePage<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    let mut grid_model = (0..COL_COUNT * ROW_COUNT)
      .into_par_iter()
      .map(|_| GameCell::Count(0))
      .collect::<Vec<_>>();

    // Spawn random bombs
    for bomb_index in index::sample(&mut thread_rng(), (COL_COUNT * ROW_COUNT) as _, 10) {
      grid_model[bomb_index] = GameCell::Bomb;
    }

    Box::new(GamePageState {
      grid_model: Arc::new(RwLock::new(grid_model)),
    })
  }
}

#[derive(Debug, Default)]
struct GamePageState {
  grid_model: Arc<RwLock<Vec<GameCell>>>,
}

impl<'a> State<'a> for GamePageState {
  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let grid_model = Arc::clone(&self.grid_model);

    Column::new()
      .children(vec![Grid {
        col_count: COL_COUNT,
        row_count: ROW_COUNT,
        gap: 2.0,
        builder: Box::new(move |index| {
          let grid_model = grid_model.read().unwrap();

          match grid_model[index as usize] {
            GameCell::Count(bomb_count) => {
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
            }
            GameCell::Bomb => ImageWidget::new("assets/avoid_the_bomb/images/bomb.png")
              .call()
              .into_widget(),
            GameCell::Flag => ImageWidget::new("assets/avoid_the_bomb/images/bomb.png")
              .call()
              .into_widget(),
          }
        }),
      }
      .into_widget()])
      .call()
      .into_widget()
  }
}
