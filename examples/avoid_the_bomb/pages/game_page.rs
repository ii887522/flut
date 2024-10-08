use crate::models::GameCell;
use flut::{
  helpers::AnimationCount,
  widgets::{
    router::Navigator, stateful_widget::State, widget::*, Button, Center, Column, Grid,
    ImageWidget, StatefulWidget, Text, Widget,
  },
};
use rand::{prelude::*, seq::index};
use rayon::prelude::*;
use skia_safe::{Color, Rect};
use std::{
  collections::{HashSet, VecDeque},
  sync::{Arc, Mutex, RwLock},
};

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
      .map(|_| GameCell::Count {
        count: 0,
        is_visible: false,
      })
      .collect::<Vec<_>>();

    // Spawn random bombs
    for bomb_index in index::sample(&mut thread_rng(), (COL_COUNT * ROW_COUNT) as _, 100) {
      grid_model[bomb_index] = GameCell::Bomb { is_visible: false };

      // Record this bomb on neighbor cells
      // Left neighbor
      // Ensure not on the left-most column of the game board to avoid wrapping to the above row
      if bomb_index % COL_COUNT as usize > 0 {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          is_visible,
        } = grid_model[bomb_index - 1]
        {
          grid_model[bomb_index - 1] = GameCell::Count {
            count: bomb_count + 1,
            is_visible,
          };
        }
      }

      // Right neighbor
      // Ensure not on the right-most column of the game board to avoid wrapping to the below row
      if (bomb_index + 1) % COL_COUNT as usize > 0 {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          is_visible,
        } = grid_model[bomb_index + 1]
        {
          grid_model[bomb_index + 1] = GameCell::Count {
            count: bomb_count + 1,
            is_visible,
          };
        }
      }

      // Top neighbor
      // Ensure not on the top-most row of the game board to avoid out of bounds error
      if bomb_index / COL_COUNT as usize > 0 {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          is_visible,
        } = grid_model[bomb_index - COL_COUNT as usize]
        {
          grid_model[bomb_index - COL_COUNT as usize] = GameCell::Count {
            count: bomb_count + 1,
            is_visible,
          };
        }
      }

      // Bottom neighbor
      // Ensure not on the bottom-most row of the game board to avoid out of bounds error
      if bomb_index + (COL_COUNT as usize) < grid_model.len() {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          is_visible,
        } = grid_model[bomb_index + COL_COUNT as usize]
        {
          grid_model[bomb_index + COL_COUNT as usize] = GameCell::Count {
            count: bomb_count + 1,
            is_visible,
          };
        }
      }

      // Top left neighbor
      // Ensure not on the left-most column of the game board to avoid wrapping to the above row
      // Ensure not on the top-most row of the game board to avoid out of bounds error
      if bomb_index % COL_COUNT as usize > 0 && bomb_index / COL_COUNT as usize > 0 {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          is_visible,
        } = grid_model[bomb_index - COL_COUNT as usize - 1]
        {
          grid_model[bomb_index - COL_COUNT as usize - 1] = GameCell::Count {
            count: bomb_count + 1,
            is_visible,
          };
        }
      }

      // Top right neighbor
      // Ensure not on the right-most column of the game board to avoid wrapping to the below row
      // Ensure not on the top-most row of the game board to avoid out of bounds error
      if (bomb_index + 1) % COL_COUNT as usize > 0 && bomb_index / COL_COUNT as usize > 0 {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          is_visible,
        } = grid_model[bomb_index - COL_COUNT as usize + 1]
        {
          grid_model[bomb_index - COL_COUNT as usize + 1] = GameCell::Count {
            count: bomb_count + 1,
            is_visible,
          };
        }
      }

      // Bottom left neighbor
      // Ensure not on the left-most column of the game board to avoid wrapping to the above row
      // Ensure not on the bottom-most row of the game board to avoid out of bounds error
      if bomb_index % COL_COUNT as usize > 0 && bomb_index + (COL_COUNT as usize) < grid_model.len()
      {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          is_visible,
        } = grid_model[bomb_index + COL_COUNT as usize - 1]
        {
          grid_model[bomb_index + COL_COUNT as usize - 1] = GameCell::Count {
            count: bomb_count + 1,
            is_visible,
          };
        }
      }

      // Bottom right neighbor
      // Ensure not on the right-most column of the game board to avoid wrapping to the below row
      // Ensure not on the bottom-most row of the game board to avoid out of bounds error
      if (bomb_index + 1) % COL_COUNT as usize > 0
        && bomb_index + (COL_COUNT as usize) < grid_model.len()
      {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          is_visible,
        } = grid_model[bomb_index + COL_COUNT as usize + 1]
        {
          grid_model[bomb_index + COL_COUNT as usize + 1] = GameCell::Count {
            count: bomb_count + 1,
            is_visible,
          };
        }
      }
    }

    Box::new(GamePageState {
      inner: Arc::new(RwLock::new(GamePageStateInner {
        grid_model,
        animation_count: AnimationCount::new(),
      })),
    })
  }
}

#[derive(Debug, Default)]
struct GamePageStateInner {
  grid_model: Vec<GameCell>,
  animation_count: AnimationCount,
}

#[derive(Debug, Default)]
struct GamePageState {
  inner: Arc<RwLock<GamePageStateInner>>,
}

impl GamePageStateInner {
  fn reveal_bomb_count(&mut self, index: u32) -> u8 {
    let index = index as usize;

    let GameCell::Count {
      count: bomb_count, ..
    } = self.grid_model[index]
    else {
      unreachable!("Caller should know index refers to bomb count before calling this method");
    };

    self.grid_model[index] = GameCell::Count {
      count: bomb_count,
      is_visible: true,
    };

    self.animation_count.incr();
    bomb_count
  }

  fn reveal_surronding(&mut self, index: u32) {
    let mut index_fifo_q = VecDeque::from_iter([index]);
    let mut visited_indices = HashSet::new();

    while let Some(index) = index_fifo_q.pop_front() {
      let bomb_count = self.reveal_bomb_count(index);
      visited_indices.insert(index);

      if bomb_count > 0 {
        continue;
      }

      // Traverse the game board in breadth-first order
      // Left neighbor
      if index % COL_COUNT as u32 > 0 && !visited_indices.contains(&(index - 1)) {
        index_fifo_q.push_back(index - 1);
      }
      // Right neighbor
      if (index + 1) % COL_COUNT as u32 > 0 && !visited_indices.contains(&(index + 1)) {
        index_fifo_q.push_back(index + 1);
      }
      // Top neighbor
      if index / COL_COUNT as u32 > 0 && !visited_indices.contains(&(index - COL_COUNT as u32)) {
        index_fifo_q.push_back(index - COL_COUNT as u32);
      }
      // Bottom neighbor
      if index + (COL_COUNT as u32) < self.grid_model.len() as _
        && !visited_indices.contains(&(index + COL_COUNT as u32))
      {
        index_fifo_q.push_back(index + COL_COUNT as u32);
      }
    }
  }

  fn reveal_all(&mut self) {
    self.grid_model.par_iter_mut().for_each(|cell| match cell {
      GameCell::Count { is_visible, .. } => *is_visible = true,
      GameCell::Bomb { is_visible } => *is_visible = true,
      GameCell::Flag => {}
    });

    self.animation_count.incr();
  }
}

impl<'a> State<'a> for GamePageState {
  fn update(&mut self, _dt: f32) -> bool {
    let mut state = self.inner.write().unwrap();

    if *state.animation_count == 0 {
      return false;
    }

    state.animation_count = AnimationCount::new();
    true
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let state_arc = Arc::clone(&self.inner);

    Column::new()
      .children(vec![Grid {
        col_count: COL_COUNT,
        row_count: ROW_COUNT,
        gap: 2.0,
        builder: Box::new(move |index| {
          let state = state_arc.read().unwrap();
          let state_arc = Arc::clone(&state_arc);

          match state.grid_model[index as usize] {
            GameCell::Count {
              count: bomb_count,
              is_visible,
            } => {
              if is_visible {
                if bomb_count > 0 {
                  Some(
                    Center {
                      child: Some(
                        Text::new()
                          .text(bomb_count.to_string())
                          .font_size(24.0)
                          .color(Color::LIGHT_GRAY)
                          .call()
                          .into_widget(),
                      ),
                    }
                    .into_widget(),
                  )
                } else {
                  None
                }
              } else {
                Some(
                  Button {
                    bg_color: Color::from_rgb(56, 56, 56),
                    border_radius: 0.0,
                    is_elevated: false,
                    is_cursor_fixed: true,
                    has_effect: false,
                    on_mouse_up: Arc::new(Mutex::new(move || {
                      let mut state = state_arc.write().unwrap();
                      let bomb_count = state.reveal_bomb_count(index);

                      if bomb_count == 0 {
                        state.reveal_surronding(index);
                      }
                    })),
                    ..Default::default()
                  }
                  .into_widget(),
                )
              }
            }
            GameCell::Bomb { is_visible } => {
              if is_visible {
                Some(
                  ImageWidget::new("assets/avoid_the_bomb/images/bomb.png")
                    .call()
                    .into_widget(),
                )
              } else {
                Some(
                  Button {
                    bg_color: Color::from_rgb(56, 56, 56),
                    border_radius: 0.0,
                    is_elevated: false,
                    is_cursor_fixed: true,
                    has_effect: false,
                    on_mouse_up: Arc::new(Mutex::new(move || {
                      let mut state = state_arc.write().unwrap();

                      // Game over. Reveal the whole game board
                      state.reveal_all();
                    })),
                    ..Default::default()
                  }
                  .into_widget(),
                )
              }
            }
            GameCell::Flag => Some(
              ImageWidget::new("assets/avoid_the_bomb/images/flag.png")
                .call()
                .into_widget(),
            ),
          }
        }),
      }
      .into_widget()])
      .call()
      .into_widget()
  }
}
