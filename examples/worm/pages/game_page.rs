use crate::{
  models::{Direction, GameCell, WormCell},
  widgets::GameOverDialog,
};
use flut::{
  boot::context,
  collections::U16SparseSet,
  helpers::{AnimationCount, Clock, ShakeAnimationSM},
  models::{AudioTask, HorizontalAlign, TextStyle},
  widgets::{
    router::Navigator, stateful_widget::State, widget::*, Column, Grid, RectWidget, Spacing,
    StatefulWidget, Text, Translation, Widget,
  },
};
use rand::prelude::*;
use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};
use skia_safe::{Color, Rect};
use std::{
  collections::VecDeque,
  sync::{Arc, Mutex, RwLock},
};

const COL_COUNT: u16 = 41;
const ROW_COUNT: u16 = 41;

#[derive(Debug)]
pub(crate) struct GamePage<'a> {
  pub(crate) navigator: Arc<Mutex<Navigator<'a>>>,
}

impl<'a> StatefulWidget<'a> for GamePage<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
      let _ = audio_tx.send(AudioTask::LoadSound("assets/worm/audio/dead.wav"));
      let _ = audio_tx.send(AudioTask::LoadSound("assets/worm/audio/eat.wav"));
    }

    Box::new(GamePageState {
      clock: Clock::new(30.0),
      navigator: Arc::clone(&self.navigator),
      inner: Arc::new(RwLock::new(GamePageStateInner::new())),
    })
  }
}

#[derive(Debug, Default, PartialEq, PartialOrd)]
struct GamePageStateInner {
  grid_model: Vec<GameCell>,
  air_indices: U16SparseSet,
  worm: VecDeque<WormCell>, // Front is head, back is tail
  next_worm_direction: Option<Direction>,
  is_worm_dead: bool,
  animation_count: AnimationCount,
  shake_animation_sm: ShakeAnimationSM,
}

#[derive(Debug, Default)]
struct GamePageState<'a> {
  clock: Clock,
  navigator: Arc<Mutex<Navigator<'a>>>,
  inner: Arc<RwLock<GamePageStateInner>>,
}

impl GamePageStateInner {
  fn new() -> Self {
    let mut this = Self::default();
    this.init();
    this
  }

  fn init(&mut self) {
    // Keep the game running
    self.animation_count.incr();

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

    let shake_animation_sm = ShakeAnimationSM::new()
      .magnitude(32.0)
      .duration(0.25)
      .call();

    self.grid_model = grid_model;
    self.air_indices = air_indices;
    self.worm = worm;
    self.next_worm_direction = None;
    self.is_worm_dead = false;
    self.shake_animation_sm = shake_animation_sm;
    self.spawn_food();
  }

  fn set_grid_cell(&mut self, index: u16, cell: GameCell) {
    let old_cell = self.grid_model[index as usize];

    if old_cell == cell {
      return;
    }

    self.grid_model[index as usize] = cell;

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

    if let GameCell::Wall | GameCell::Worm = self.grid_model[new_head.position as usize] {
      if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
        let _ = audio_tx.send(AudioTask::PlaySound("assets/worm/audio/dead.wav"));
      }

      self.is_worm_dead = true;

      // Game can stop running after game over
      self.animation_count = AnimationCount::new();

      self.shake_animation_sm.shake();
      return;
    } else if let GameCell::Food = self.grid_model[new_head.position as usize] {
      if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
        let _ = audio_tx.send(AudioTask::PlaySound("assets/worm/audio/eat.wav"));
      }

      self.spawn_food();
    } else {
      let tail = self.worm.pop_back().unwrap();
      self.set_grid_cell(tail.position, GameCell::Air);
    }

    self.worm.push_front(new_head);
    self.set_grid_cell(new_head.position, GameCell::Worm);
  }
}

impl<'a> State<'a> for GamePageState<'a> {
  fn process_event(&mut self, event: &Event) -> bool {
    let mut state = self.inner.write().unwrap();

    if state.is_worm_dead {
      // Don't consume the event because dialog buttons might need to listen to it
      return false;
    }

    let worm_head = state.worm.front().unwrap();

    match event {
      Event::KeyDown {
        keycode: Some(Keycode::W | Keycode::Up),
        ..
      } => {
        if worm_head.direction != Direction::Down {
          state.next_worm_direction = Some(Direction::Up);
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::D | Keycode::Right),
        ..
      } => {
        if worm_head.direction != Direction::Left {
          state.next_worm_direction = Some(Direction::Right);
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::S | Keycode::Down),
        ..
      } => {
        if worm_head.direction != Direction::Up {
          state.next_worm_direction = Some(Direction::Down);
        }
      }
      Event::KeyDown {
        keycode: Some(Keycode::A | Keycode::Left),
        ..
      } => {
        if worm_head.direction != Direction::Right {
          state.next_worm_direction = Some(Direction::Left);
        }
      }
      _ => {}
    }

    // Don't consume the event because dialog buttons might need to listen to it
    false
  }

  fn update(&mut self, dt: f32) -> bool {
    let mut state = self.inner.write().unwrap();

    if !self.clock.update(dt) || state.is_worm_dead {
      return state.shake_animation_sm.update(dt);
    }

    if let Some(next_worm_direction) = state.next_worm_direction.take() {
      let worm_head = state.worm.front_mut().unwrap();
      worm_head.direction = next_worm_direction;
    }

    state.move_worm();
    true
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let state_arc_1 = Arc::clone(&self.inner);
    let state_arc_2 = Arc::clone(&self.inner);
    let state = self.inner.read().unwrap();
    let score = state.worm.len() - 1;
    let navigator = Arc::clone(&self.navigator);

    Column::new()
      .children(
        vec![
          Some(
            Translation {
              translation: state.shake_animation_sm.get_current_translation(),
              child: Column::new()
                .align(HorizontalAlign::Center)
                .children(vec![
                  Spacing {
                    height: 16.0,
                    ..Default::default()
                  }
                  .into_widget(),
                  Text::new()
                    .text(score.to_string())
                    .style(TextStyle {
                      color: Color::WHITE,
                      font_size: 48.0,
                      ..Default::default()
                    })
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
                      let state = state_arc_1.read().unwrap();

                      Some(
                        RectWidget {
                          color: match state.grid_model[index as usize] {
                            GameCell::Air => Color::from_rgb(56, 56, 56),
                            GameCell::Worm => Color::from_rgb(243, 125, 121),
                            GameCell::Wall => Color::RED,
                            GameCell::Food => Color::GREEN,
                          },
                          ..Default::default()
                        }
                        .into_widget(),
                      )
                    }),
                  }
                  .into_widget(),
                ])
                .call()
                .into_widget(),
            }
            .into_widget(),
          ),
          if state.is_worm_dead {
            Some(
              GameOverDialog {
                navigator,
                score: score as _,
                on_ok: Arc::new(Mutex::new(move || {
                  // Restart the game
                  let mut state = state_arc_2.write().unwrap();
                  state.init();
                })),
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
