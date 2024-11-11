use crate::models::{Direction, GameCell, WormCell};
use atomic_refcell::AtomicRefCell;
use flut::{
  models::{FontCfg, HorizontalAlign},
  widgets::{widget::*, BuilderWidget, Column, Grid, RectWidget, Spacing, Text, Widget},
};
use sdl2::event::Event;
use skia_safe::{Color, Rect};
use std::{collections::VecDeque, sync::Arc};

const COL_COUNT: u32 = 41;
const ROW_COUNT: u32 = 41;
const CELL_COUNT: u32 = COL_COUNT * ROW_COUNT;

pub(crate) struct GamePage {
  grid_model: Arc<AtomicRefCell<[GameCell; CELL_COUNT as usize]>>,
  worm: VecDeque<WormCell>, // Front is tail, back is head
}

impl GamePage {
  pub(crate) fn new() -> Self {
    // Given an empty game board
    let mut grid_model = [GameCell::Air; CELL_COUNT as usize];

    for i in 0..COL_COUNT {
      // Put wall at the top-most row
      grid_model[i as usize] = GameCell::Wall;

      // Put wall at the bottom-most row
      grid_model[(CELL_COUNT - 1 - i) as usize] = GameCell::Wall;
    }

    for i in 0..ROW_COUNT {
      // Put wall at the left-most column
      grid_model[(i * COL_COUNT) as usize] = GameCell::Wall;

      // Put wall at the right-most column
      grid_model[(CELL_COUNT - 1 - i * COL_COUNT) as usize] = GameCell::Wall;
    }

    // Put a worm on the center
    grid_model[(CELL_COUNT >> 1) as usize] = GameCell::Worm;

    let worm_cell = WormCell {
      position: (CELL_COUNT >> 1) as _,
      direction: Direction::rand(),
    };

    Self {
      grid_model: Arc::new(AtomicRefCell::new(grid_model)),
      worm: VecDeque::from_iter([worm_cell]),
    }
  }
}

impl<'a> BuilderWidget<'a> for GamePage {
  fn process_event(&mut self, _event: Event) {}

  fn update(&mut self, _dt: f32) -> bool {
    let old_tail = self.worm.pop_front().unwrap();

    let new_head = WormCell {
      position: match old_tail.direction {
        Direction::Up => old_tail.position - COL_COUNT as u16,
        Direction::Down => old_tail.position + COL_COUNT as u16,
        Direction::Left => old_tail.position - 1,
        Direction::Right => old_tail.position + 1,
      },
      direction: old_tail.direction,
    };

    self.worm.push_back(new_head);
    true
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let grid_model = Arc::clone(&self.grid_model);

    Column {
      align: HorizontalAlign::Center,
      children: vec![
        Spacing {
          height: 16.0,
          ..Default::default()
        }
        .into_widget(),
        Text::new()
          .text("0")
          .font_cfg(FontCfg {
            font_size: 48,
            ..Default::default()
          })
          .color(Color::WHITE)
          .call()
          .into_widget(),
        Spacing {
          height: 16.0,
          ..Default::default()
        }
        .into_widget(),
        Grid {
          col_count: COL_COUNT,
          row_count: ROW_COUNT,
          gap: 2.0,
          builder: Box::new(move |index| {
            RectWidget {
              color: match grid_model.borrow()[index as usize] {
                GameCell::Air => Color::DARK_GRAY,
                GameCell::Worm => Color::from_rgb(243, 125, 121),
                GameCell::Wall => Color::RED,
                GameCell::Food => Color::GREEN,
              },
            }
            .into_widget()
          }),
        }
        .into_widget(),
      ],
    }
    .into_widget()
  }
}
