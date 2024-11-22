use crate::models::{Direction, GameCell, WormCell};
use atomic_refcell::AtomicRefCell;
use flut::{
  helpers::Clock,
  models::{FontCfg, HorizontalAlign},
  widgets::{widget::*, BuilderWidget, Column, Grid, RectWidget, Spacing, Text, Widget},
};
use sdl2::{event::Event, keyboard::Keycode};
use skia_safe::{Color, Rect};
use std::{
  collections::{HashSet, VecDeque},
  sync::Arc,
};

const COL_COUNT: u16 = 41;
const ROW_COUNT: u16 = 41;
const CELL_COUNT: u16 = COL_COUNT * ROW_COUNT;

pub(crate) struct GamePage {
  grid_model: Arc<AtomicRefCell<[GameCell; CELL_COUNT as usize]>>,
  air_indices: HashSet<u16>,
  worm: VecDeque<WormCell>, // Front is tail, back is head
  clock: Clock,
  is_worm_dead: bool,
}

impl GamePage {
  pub(crate) fn new() -> Self {
    // Given an empty game board
    let mut this = Self {
      grid_model: Arc::new(AtomicRefCell::new([GameCell::Air; CELL_COUNT as usize])),
      air_indices: (0..CELL_COUNT).collect(),
      worm: VecDeque::new(),
      clock: Clock::new(30.0),
      is_worm_dead: false,
    };

    for i in 0..COL_COUNT {
      // Put wall at the top-most row
      this.set_cell(i, GameCell::Wall);

      // Put wall at the bottom-most row
      this.set_cell(CELL_COUNT - 1 - i, GameCell::Wall);
    }

    for i in 0..ROW_COUNT {
      // Put wall at the left-most column
      this.set_cell(i * COL_COUNT, GameCell::Wall);

      // Put wall at the right-most column
      this.set_cell(CELL_COUNT - 1 - i * COL_COUNT, GameCell::Wall);
    }

    // Put a worm on the center
    this.set_cell(CELL_COUNT >> 1, GameCell::Worm);

    let worm_cell = WormCell {
      position: (CELL_COUNT >> 1) as _,
      direction: Direction::rand(),
    };

    this.worm.push_back(worm_cell);
    this.spawn_food();
    this
  }

  fn set_cell(&mut self, index: u16, cell: GameCell) {
    let mut grid_model = self.grid_model.borrow_mut();
    grid_model[index as usize] = cell;

    if cell == GameCell::Air {
      self.air_indices.insert(index);
    } else {
      self.air_indices.remove(&index);
    }
  }

  fn grow_worm(&mut self) {
    let old_worm_head = self.worm.back().unwrap();

    let new_worm_head = WormCell {
      position: match old_worm_head.direction {
        Direction::Up => old_worm_head.position - COL_COUNT,
        Direction::Right => old_worm_head.position + 1,
        Direction::Down => old_worm_head.position + COL_COUNT,
        Direction::Left => old_worm_head.position - 1,
      },
      direction: old_worm_head.direction,
    };

    self.worm.push_back(new_worm_head);
    self.set_cell(new_worm_head.position, GameCell::Worm);
  }

  fn move_worm(&mut self) {
    self.grow_worm();

    let old_tail = self.worm.pop_front().unwrap();
    self.set_cell(old_tail.position, GameCell::Air);
  }

  fn spawn_food(&mut self) {
    // Put a food at a random position
    let food_index = *fastrand::choice(&self.air_indices).unwrap();
    self.set_cell(food_index, GameCell::Food);
  }
}

impl<'a> BuilderWidget<'a> for GamePage {
  fn process_event(&mut self, event: &Event) {
    if self.is_worm_dead {
      return;
    }

    let worm_head = self.worm.back_mut().unwrap();

    match event {
      Event::KeyDown {
        keycode: Some(Keycode::W | Keycode::Up),
        ..
      } => {
        if worm_head.direction != Direction::Down {
          worm_head.direction = Direction::Up;
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::D | Keycode::Right),
        ..
      } => {
        if worm_head.direction != Direction::Left {
          worm_head.direction = Direction::Right;
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::S | Keycode::Down),
        ..
      } => {
        if worm_head.direction != Direction::Up {
          worm_head.direction = Direction::Down;
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::A | Keycode::Left),
        ..
      } => {
        if worm_head.direction != Direction::Right {
          worm_head.direction = Direction::Left;
        }
      }
      _ => {}
    }
  }

  fn update(&mut self, dt: f32) -> bool {
    if !self.clock.update(dt) {
      return true;
    }

    let old_worm_head = self.worm.back().unwrap();
    let grid_model = self.grid_model.borrow();

    match old_worm_head.direction {
      Direction::Up => match grid_model[(old_worm_head.position - COL_COUNT) as usize] {
        GameCell::Worm | GameCell::Wall => {
          self.is_worm_dead = true;
          return false;
        }
        GameCell::Food => {
          drop(grid_model);
          self.grow_worm();
          self.spawn_food();
        }
        GameCell::Air => {
          drop(grid_model);
          self.move_worm();
        }
      },
      Direction::Right => match grid_model[(old_worm_head.position + 1) as usize] {
        GameCell::Worm | GameCell::Wall => {
          self.is_worm_dead = true;
          return false;
        }
        GameCell::Food => {
          drop(grid_model);
          self.grow_worm();
          self.spawn_food();
        }
        GameCell::Air => {
          drop(grid_model);
          self.move_worm();
        }
      },
      Direction::Down => match grid_model[(old_worm_head.position + COL_COUNT) as usize] {
        GameCell::Worm | GameCell::Wall => {
          self.is_worm_dead = true;
          return false;
        }
        GameCell::Food => {
          drop(grid_model);
          self.grow_worm();
          self.spawn_food();
        }
        GameCell::Air => {
          drop(grid_model);
          self.move_worm();
        }
      },
      Direction::Left => match grid_model[(old_worm_head.position - 1) as usize] {
        GameCell::Worm | GameCell::Wall => {
          self.is_worm_dead = true;
          return false;
        }
        GameCell::Food => {
          drop(grid_model);
          self.grow_worm();
          self.spawn_food();
        }
        GameCell::Air => {
          drop(grid_model);
          self.move_worm();
        }
      },
    }

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
          .text((self.worm.len() - 1).to_string())
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
          col_count: COL_COUNT as _,
          row_count: ROW_COUNT as _,
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
