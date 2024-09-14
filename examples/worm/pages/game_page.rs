use crate::models::{Direction, GameCell, WormCell};
use flut::{
  boot::context,
  collections::U16SparseSet,
  helpers::Clock,
  models::{icon_name, AudioTask, HorizontalAlign},
  widgets::{
    stateful_widget::State, widget::*, Column, Dialog, Grid, RectWidget, Spacing, StatefulWidget,
    Text, Widget,
  },
};
use rand::prelude::*;
use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};
use skia_safe::{Color, Rect};
use std::{
  collections::VecDeque,
  process,
  sync::{atomic::Ordering, Arc, RwLock},
};

const COL_COUNT: u16 = 41;
const ROW_COUNT: u16 = 41;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GamePage;

impl<'a> StatefulWidget<'a> for GamePage {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    context::AUDIO_TX.with(|audio_tx| {
      if let Some(audio_tx) = audio_tx.get() {
        let _ = audio_tx.send(AudioTask::LoadSound("assets/audio/dead.wav"));
        let _ = audio_tx.send(AudioTask::LoadSound("assets/audio/eat.wav"));
      }
    });

    let mut state = GamePageState {
      clock: Clock::new(30.0),
      ..Default::default()
    };

    state.init();
    Box::new(state)
  }
}

#[derive(Debug, Default)]
struct GamePageState {
  grid_model: Arc<RwLock<Vec<GameCell>>>,
  air_indices: U16SparseSet,
  worm: VecDeque<WormCell>, // Front is head, back is tail
  clock: Clock,
  next_worm_direction: Option<Direction>,
  is_worm_dead: bool,
}

impl GamePageState {
  fn init(&mut self) {
    // Keep the game running
    context::ANIMATION_COUNT.fetch_add(1, Ordering::Relaxed);

    let grid_model = (0..COL_COUNT * ROW_COUNT)
      .into_par_iter()
      .map(|index| {
        // Put a worm on the center
        if index == (COL_COUNT * ROW_COUNT) >> 1 {
          GameCell::Worm

        // Put top walls
        } else if index < COL_COUNT
        // Put bottom walls
        || ((COL_COUNT - 1) * ROW_COUNT..COL_COUNT * ROW_COUNT).contains(&index)
        // Put left walls
        || index % COL_COUNT == 0
        // Put right walls
        || (index + 1) % COL_COUNT == 0
        {
          GameCell::Wall
        } else {
          GameCell::Air
        }
      })
      .collect::<Vec<_>>();

    let air_indices = U16SparseSet::from_par_iter(grid_model.par_iter().enumerate().filter_map(
      |(index, &cell)| {
        if cell == GameCell::Air {
          Some(index as _)
        } else {
          None
        }
      },
    ));

    let mut worm = VecDeque::with_capacity(((COL_COUNT - 2) * (ROW_COUNT - 2)) as _);

    worm.push_front(WormCell {
      position: (COL_COUNT * ROW_COUNT) >> 1,
      direction: random(),
    });

    self.grid_model = Arc::new(RwLock::new(grid_model));
    self.air_indices = air_indices;
    self.worm = worm;
    self.next_worm_direction = None;
    self.is_worm_dead = false;
    self.spawn_food();
  }

  fn set_grid_cell(&mut self, index: u16, cell: GameCell) {
    let mut grid_model = self.grid_model.write().unwrap();
    let old_cell = grid_model[index as usize];

    if old_cell == cell {
      return;
    }

    grid_model[index as usize] = cell;

    if old_cell == GameCell::Air {
      self.air_indices.swap_remove(index);
    } else if cell == GameCell::Air {
      self.air_indices.push(index);
    }
  }

  fn spawn_food(&mut self) {
    let food_index = self.air_indices.random().unwrap();
    self.set_grid_cell(food_index, GameCell::Food);
  }

  fn move_worm(&mut self) {
    let head = self.worm.front().unwrap();

    let new_head = WormCell {
      position: match head.direction {
        Direction::Up => head.position - COL_COUNT,
        Direction::Right => head.position + 1,
        Direction::Down => head.position + COL_COUNT,
        Direction::Left => head.position - 1,
      },
      direction: head.direction,
    };

    {
      let grid_model = self.grid_model.read().unwrap();

      if let GameCell::Wall | GameCell::Worm = grid_model[new_head.position as usize] {
        context::AUDIO_TX.with(|audio_tx| {
          if let Some(audio_tx) = audio_tx.get() {
            let _ = audio_tx.send(AudioTask::PlaySound("assets/audio/dead.wav"));
          }
        });

        self.is_worm_dead = true;

        // Game can stop running after game over
        context::ANIMATION_COUNT.fetch_sub(1, Ordering::Relaxed);
        return;
      } else if let GameCell::Food = grid_model[new_head.position as usize] {
        drop(grid_model);

        context::AUDIO_TX.with(|audio_tx| {
          if let Some(audio_tx) = audio_tx.get() {
            let _ = audio_tx.send(AudioTask::PlaySound("assets/audio/eat.wav"));
          }
        });

        self.spawn_food();
      } else {
        drop(grid_model);
        let tail = self.worm.pop_back().unwrap();
        self.set_grid_cell(tail.position, GameCell::Air);
      }
    }

    self.worm.push_front(new_head);
    self.set_grid_cell(new_head.position, GameCell::Worm);
  }
}

impl<'a> State<'a> for GamePageState {
  fn process_event(&mut self, event: &Event) -> bool {
    if self.is_worm_dead {
      // Don't consume the event because dialog buttons might need to listen to it
      return false;
    }

    let worm_head = self.worm.front().unwrap();

    match event {
      Event::KeyDown {
        keycode: Some(Keycode::W | Keycode::Up),
        ..
      } => {
        if worm_head.direction != Direction::Down {
          self.next_worm_direction = Some(Direction::Up);
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::D | Keycode::Right),
        ..
      } => {
        if worm_head.direction != Direction::Left {
          self.next_worm_direction = Some(Direction::Right);
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::S | Keycode::Down),
        ..
      } => {
        if worm_head.direction != Direction::Up {
          self.next_worm_direction = Some(Direction::Down);
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::A | Keycode::Left),
        ..
      } => {
        if worm_head.direction != Direction::Right {
          self.next_worm_direction = Some(Direction::Left);
        }
      }
      _ => {}
    }

    // Don't consume the event because dialog buttons might need to listen to it
    false
  }

  fn update(&mut self, dt: f32) -> bool {
    if !self.clock.update(dt) || self.is_worm_dead {
      return false;
    }

    if let Some(next_worm_direction) = self.next_worm_direction.take() {
      let worm_head = self.worm.front_mut().unwrap();
      worm_head.direction = next_worm_direction;
    }

    self.move_worm();
    true
  }

  fn build(&self, _constraint: Rect) -> Widget<'a> {
    let grid_model = Arc::clone(&self.grid_model);
    let score = self.worm.len() - 1;

    Column::new()
      .align(HorizontalAlign::Center)
      .children(
        vec![
          Some(
            Spacing {
              height: 16.0,
              ..Default::default()
            }
            .into_widget(),
          ),
          Some(
            Text::new()
              .text(score.to_string())
              .color(Color::WHITE)
              .font_size(48.0)
              .call()
              .into_widget(),
          ),
          Some(
            Spacing {
              height: 16.0,
              ..Default::default()
            }
            .into_widget(),
          ),
          Some(
            Grid {
              col_count: COL_COUNT,
              row_count: ROW_COUNT,
              gap: 2.0,
              builder: Box::new(move |index| {
                let grid_model = grid_model.read().unwrap();

                RectWidget {
                  color: match grid_model[index as usize] {
                    GameCell::Air => Color::from_rgb(56, 56, 56),
                    GameCell::Worm => Color::from_rgb(243, 125, 121),
                    GameCell::Wall => Color::RED,
                    GameCell::Food => Color::GREEN,
                  },
                  ..Default::default()
                }
                .into_widget()
              }),
            }
            .into_widget(),
          ),
          if self.is_worm_dead {
            Some(
              Dialog {
                color: Color::from_rgb(255, 128, 128),
                header_icon: icon_name::SKULL,
                header_title: "You Died...".to_string(),
                has_ok: true,
                close_icon: icon_name::SENTIMENT_DISSATISFIED,
                close_label: "Give Up".to_string(),
                ok_icon: icon_name::RESTART_ALT,
                ok_label: "Try Again".to_string(),
                on_close: Some(Box::new(|| process::exit(0))),
                on_ok: Some(Box::new(|| {
                  // Restart the game
                  // todo: self.init();
                })),
                body: Some(
                  Text::new()
                    .text(format!(
                      "You ate {score} green apple{}. Want to try again?",
                      if score != 1 { "s" } else { "" },
                    ))
                    .font_size(24.0)
                    .call()
                    .into_widget(),
                ),
                ..Default::default()
              }
              .into_widget(),
            )
          } else {
            None
          },
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>(),
      )
      .call()
      .into_widget()
  }
}
