use crate::{
  i18n::I18N,
  models::{Difficulty, GameCell, GameCellState, GameState},
  widgets::{BackConfirmDialog, GameOverDialog, YouWonDialog},
};
use atomic_refcell::AtomicRefCell;
use flut::{
  boot::context,
  helpers::{AnimationCount, ShakeAnimationSM},
  models::{icon_name, AudioReq, TextStyle},
  widgets::{
    bar::{BarButton, TitleStyle},
    router::Navigator,
    stateful_widget::State,
    widget::*,
    Bar, Button, Center, Column, Grid, ImageButton, ImageWidget, StatefulWidget, Text, Translation,
    Widget,
  },
};
use rand::{prelude::*, seq::index};
use rayon::prelude::*;
use skia_safe::{Color, Rect};
use std::{
  collections::{HashSet, VecDeque},
  sync::{Arc, RwLock},
  thread,
  time::Duration,
};

const COL_COUNT: u16 = 31;
const ROW_COUNT: u16 = 31;

#[derive(Default)]
pub(crate) struct GamePage<'a> {
  pub(crate) navigator: Arc<AtomicRefCell<Navigator<'a>>>,
  pub(crate) difficulty: Difficulty,
}

impl<'a> StatefulWidget<'a> for GamePage<'a> {
  fn new_state(&mut self) -> Box<dyn State<'a> + 'a> {
    if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
      let _ = audio_tx.send(AudioReq::LoadSound("assets/avoid_the_bomb/audio/dig.wav"));
      let _ = audio_tx.send(AudioReq::LoadSound("assets/avoid_the_bomb/audio/won.wav"));

      let _ = audio_tx.send(AudioReq::LoadSound(
        "assets/avoid_the_bomb/audio/explode.wav",
      ));

      let _ = audio_tx.send(AudioReq::LoadSound("assets/avoid_the_bomb/audio/click.wav"));
    }

    Box::new(GamePageState {
      navigator: Arc::clone(&self.navigator),
      inner: Arc::new(RwLock::new(GamePageStateInner::new(self.difficulty))),
    })
  }
}

#[derive(Default)]
struct GamePageStateInner {
  difficulty: Difficulty,
  grid_model: Vec<GameCell>,
  flagged_bomb_count: u32,
  visible_digit_count: u32,
  game_state: GameState,
  animation_count: AnimationCount,
  shake_animation_sm: ShakeAnimationSM,
}

struct GamePageState<'a> {
  navigator: Arc<AtomicRefCell<Navigator<'a>>>,
  inner: Arc<RwLock<GamePageStateInner>>,
}

impl GamePageStateInner {
  fn new(difficulty: Difficulty) -> Self {
    let mut this = Self {
      difficulty,
      ..Default::default()
    };

    this.init();
    this
  }

  fn init(&mut self) {
    // Keep the game running
    self.animation_count.incr();

    let mut grid_model = (0..COL_COUNT * ROW_COUNT)
      .into_par_iter()
      .map(|_| GameCell::Count {
        count: 0,
        state: GameCellState::Hidden,
      })
      .collect::<Vec<_>>();

    // Spawn random bombs
    for bomb_index in index::sample(
      &mut thread_rng(),
      (COL_COUNT * ROW_COUNT) as _,
      self.difficulty.get_bomb_count(),
    ) {
      grid_model[bomb_index] = GameCell::Bomb {
        state: GameCellState::Hidden,
      };

      // Record this bomb on neighbor cells
      // Left neighbor
      // Ensure not on the left-most column of the game board to avoid wrapping to the above row
      if bomb_index % COL_COUNT as usize > 0 {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          state,
        } = grid_model[bomb_index - 1]
        {
          grid_model[bomb_index - 1] = GameCell::Count {
            count: bomb_count + 1,
            state,
          };
        }
      }

      // Right neighbor
      // Ensure not on the right-most column of the game board to avoid wrapping to the below row
      if (bomb_index + 1) % COL_COUNT as usize > 0 {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          state,
        } = grid_model[bomb_index + 1]
        {
          grid_model[bomb_index + 1] = GameCell::Count {
            count: bomb_count + 1,
            state,
          };
        }
      }

      // Top neighbor
      // Ensure not on the top-most row of the game board to avoid out of bounds error
      if bomb_index / COL_COUNT as usize > 0 {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          state,
        } = grid_model[bomb_index - COL_COUNT as usize]
        {
          grid_model[bomb_index - COL_COUNT as usize] = GameCell::Count {
            count: bomb_count + 1,
            state,
          };
        }
      }

      // Bottom neighbor
      // Ensure not on the bottom-most row of the game board to avoid out of bounds error
      if bomb_index + (COL_COUNT as usize) < grid_model.len() {
        // Dont override spawned bomb
        if let GameCell::Count {
          count: bomb_count,
          state,
        } = grid_model[bomb_index + COL_COUNT as usize]
        {
          grid_model[bomb_index + COL_COUNT as usize] = GameCell::Count {
            count: bomb_count + 1,
            state,
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
          state,
        } = grid_model[bomb_index - COL_COUNT as usize - 1]
        {
          grid_model[bomb_index - COL_COUNT as usize - 1] = GameCell::Count {
            count: bomb_count + 1,
            state,
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
          state,
        } = grid_model[bomb_index - COL_COUNT as usize + 1]
        {
          grid_model[bomb_index - COL_COUNT as usize + 1] = GameCell::Count {
            count: bomb_count + 1,
            state,
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
          state,
        } = grid_model[bomb_index + COL_COUNT as usize - 1]
        {
          grid_model[bomb_index + COL_COUNT as usize - 1] = GameCell::Count {
            count: bomb_count + 1,
            state,
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
          state,
        } = grid_model[bomb_index + COL_COUNT as usize + 1]
        {
          grid_model[bomb_index + COL_COUNT as usize + 1] = GameCell::Count {
            count: bomb_count + 1,
            state,
          };
        }
      }
    }

    let shake_animation_sm = ShakeAnimationSM::new().magnitude(32.0).duration(0.5).call();

    self.grid_model = grid_model;
    self.flagged_bomb_count = 0;
    self.visible_digit_count = 0;
    self.game_state = GameState::Playing;
    self.shake_animation_sm = shake_animation_sm;
  }

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
      state: GameCellState::Visible,
    };

    self.visible_digit_count += 1;
    self.animation_count.incr();
    bomb_count
  }

  fn reveal_surronding(&mut self, index: u32) {
    let mut index_fifo_q = VecDeque::from_iter([index]);
    let mut covered_indices = HashSet::<_>::from_iter([index]);

    while let Some(index) = index_fifo_q.pop_front() {
      let bomb_count = self.reveal_bomb_count(index);

      if bomb_count > 0 {
        continue;
      }

      // Traverse the game board in breadth-first order
      // Left neighbor
      if index % COL_COUNT as u32 > 0 && !covered_indices.contains(&(index - 1)) {
        index_fifo_q.push_back(index - 1);
        covered_indices.insert(index - 1);
      }
      // Right neighbor
      if (index + 1) % COL_COUNT as u32 > 0 && !covered_indices.contains(&(index + 1)) {
        index_fifo_q.push_back(index + 1);
        covered_indices.insert(index + 1);
      }
      // Top neighbor
      if index / COL_COUNT as u32 > 0 && !covered_indices.contains(&(index - COL_COUNT as u32)) {
        index_fifo_q.push_back(index - COL_COUNT as u32);
        covered_indices.insert(index - COL_COUNT as u32);
      }
      // Bottom neighbor
      if index + (COL_COUNT as u32) < self.grid_model.len() as _
        && !covered_indices.contains(&(index + COL_COUNT as u32))
      {
        index_fifo_q.push_back(index + COL_COUNT as u32);
        covered_indices.insert(index + COL_COUNT as u32);
      }
    }
  }

  fn reveal_all(&mut self) {
    self.grid_model.par_iter_mut().for_each(|cell| match cell {
      GameCell::Count { state, .. } => *state = GameCellState::Visible,
      GameCell::Bomb { state } => *state = GameCellState::Visible,
    });

    self.animation_count.incr();
  }

  fn set_cell_state(&mut self, index: u32, cell_state: GameCellState) {
    match &mut self.grid_model[index as usize] {
      GameCell::Count { state, .. } => *state = cell_state,
      GameCell::Bomb { state } => *state = cell_state,
    }

    self.animation_count.incr();
  }

  fn set_game_state(&mut self, game_state: GameState) {
    self.game_state = game_state;
    self.animation_count.incr();
  }

  /// All digits are revealed and All bombs are marked
  fn is_done(&self) -> bool {
    self.visible_digit_count + self.flagged_bomb_count == (COL_COUNT * ROW_COUNT) as u32
  }
}

impl<'a> State<'a> for GamePageState<'a> {
  fn update(&mut self, dt: f32) -> bool {
    let mut state = self.inner.write().unwrap();
    let is_dirty = state.shake_animation_sm.update(dt);

    if *state.animation_count == 0 {
      return is_dirty;
    }

    state.animation_count = AnimationCount::new();
    true
  }

  fn build(&mut self, _constraint: Rect) -> Widget<'a> {
    let state_arc_1 = Arc::clone(&self.inner);
    let state_arc_2 = Arc::clone(&self.inner);
    let state_arc_3 = Arc::clone(&self.inner);
    let navigator = Arc::clone(&self.navigator);
    let state = self.inner.read().unwrap();

    Column::new()
      .children(
        vec![
          Some(
            Translation {
              translation: state.shake_animation_sm.get_current_translation(),
              child: Column::new()
                .children(vec![
                  Bar {
                    color: Color::LIGHT_GRAY,
                    is_elevated: false,
                    leading_btn: BarButton {
                      is_enabled: state.game_state == GameState::Playing,
                      icon: icon_name::ARROW_BACK,
                      icon_color: Color::from_rgb(128, 0, 0),
                      on_mouse_up: Arc::new(AtomicRefCell::new(move || {
                        // Pop up back confirm dialog
                        let mut state = state_arc_3.write().unwrap();
                        state.set_game_state(GameState::Pause);
                      })),
                    },
                    title: I18N.with(|i18n| i18n.t("avoid_the_bomb").call()),
                    title_style: TitleStyle {
                      font_family: I18N.with(|i18n| i18n.get_default_font_family()),
                      ..Default::default()
                    },
                    ..Default::default()
                  }
                  .into_widget(),
                  Grid {
                    col_count: COL_COUNT,
                    row_count: ROW_COUNT,
                    gap: 2.0,
                    builder: Box::new(move |index| {
                      let state = state_arc_1.read().unwrap();
                      let state_arc_1 = Arc::clone(&state_arc_1);
                      let state_arc_2 = Arc::clone(&state_arc_1);

                      match state.grid_model[index as usize] {
                        GameCell::Count {
                          count: bomb_count,
                          state: cell_state,
                        } => match cell_state {
                          GameCellState::Hidden => Some(
                            Button {
                              is_enabled: state.game_state == GameState::Playing,
                              bg_color: Color::from_rgb(56, 56, 56),
                              border_radius: 0.0,
                              is_elevated: false,
                              is_cursor_fixed: true,
                              has_effect: false,
                              on_mouse_up: Arc::new(AtomicRefCell::new(move || {
                                if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
                                  let _ = audio_tx.send(AudioReq::PlaySound(
                                    "assets/avoid_the_bomb/audio/dig.wav",
                                  ));
                                }

                                let state_arc = Arc::clone(&state_arc_1);
                                let mut state = state_arc_1.write().unwrap();
                                state.reveal_surronding(index);

                                // Win condition
                                if !state.is_done() {
                                  return;
                                }

                                rayon::spawn(move || {
                                  thread::sleep(Duration::from_secs(1));

                                  if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
                                    let _ = audio_tx.send(AudioReq::PlaySound(
                                      "assets/avoid_the_bomb/audio/won.wav",
                                    ));
                                  }

                                  let mut state = state_arc.write().unwrap();
                                  state.set_game_state(GameState::Won);
                                });
                              })),
                              on_right_mouse_up: Arc::new(AtomicRefCell::new(move || {
                                if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
                                  let _ = audio_tx.send(AudioReq::PlaySound(
                                    "assets/avoid_the_bomb/audio/click.wav",
                                  ));
                                }

                                let mut state = state_arc_2.write().unwrap();
                                state.set_cell_state(index, GameCellState::Flagged);
                              })),
                              ..Default::default()
                            }
                            .into_widget(),
                          ),
                          GameCellState::Visible => {
                            if bomb_count > 0 {
                              Some(
                                Center {
                                  child: Some(
                                    Text::new()
                                      .text(bomb_count.to_string())
                                      .style(TextStyle {
                                        font_size: 24.0,
                                        color: Color::LIGHT_GRAY,
                                        ..Default::default()
                                      })
                                      .call()
                                      .into_widget(),
                                  ),
                                }
                                .into_widget(),
                              )
                            } else {
                              None
                            }
                          }
                          GameCellState::Flagged => Some(
                            ImageButton {
                              is_enabled: state.game_state == GameState::Playing,
                              file_path: "assets/avoid_the_bomb/images/flag.png",
                              on_right_mouse_up: Arc::new(AtomicRefCell::new(move || {
                                if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
                                  let _ = audio_tx.send(AudioReq::PlaySound(
                                    "assets/avoid_the_bomb/audio/click.wav",
                                  ));
                                }

                                let mut state = state_arc_1.write().unwrap();
                                state.set_cell_state(index, GameCellState::Hidden);
                              })),
                              ..Default::default()
                            }
                            .into_widget(),
                          ),
                        },
                        GameCell::Bomb { state: cell_state } => {
                          match cell_state {
                            GameCellState::Hidden => {
                              Some(
                                Button {
                                  is_enabled: state.game_state == GameState::Playing,
                                  bg_color: Color::from_rgb(56, 56, 56),
                                  border_radius: 0.0,
                                  is_elevated: false,
                                  is_cursor_fixed: true,
                                  has_effect: false,
                                  on_mouse_up: Arc::new(AtomicRefCell::new(move || {
                                    if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
                                      let _ = audio_tx.send(AudioReq::PlaySound(
                                        "assets/avoid_the_bomb/audio/explode.wav",
                                      ));
                                    }

                                    let state_arc = Arc::clone(&state_arc_1);
                                    let mut state = state_arc_1.write().unwrap();

                                    // Game over. Reveal the whole game board
                                    state.reveal_all();
                                    state.shake_animation_sm.shake();

                                    rayon::spawn(move || {
                                      thread::sleep(Duration::from_secs(1));
                                      let mut state = state_arc.write().unwrap();
                                      state.set_game_state(GameState::Dead);
                                    });
                                  })),
                                  on_right_mouse_up: Arc::new(AtomicRefCell::new(move || {
                                    if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
                                      let _ = audio_tx.send(AudioReq::PlaySound(
                                        "assets/avoid_the_bomb/audio/click.wav",
                                      ));
                                    }

                                    let state_arc = Arc::clone(&state_arc_2);
                                    let mut state = state_arc_2.write().unwrap();
                                    state.set_cell_state(index, GameCellState::Flagged);
                                    state.flagged_bomb_count += 1;

                                    // Win condition
                                    if !state.is_done() {
                                      return;
                                    }

                                    rayon::spawn(move || {
                                      thread::sleep(Duration::from_secs(1));

                                      if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
                                        let _ = audio_tx.send(AudioReq::PlaySound(
                                          "assets/avoid_the_bomb/audio/won.wav",
                                        ));
                                      }

                                      let mut state = state_arc.write().unwrap();
                                      state.set_game_state(GameState::Won);
                                    });
                                  })),
                                  ..Default::default()
                                }
                                .into_widget(),
                              )
                            }
                            GameCellState::Visible => Some(
                              ImageWidget::new("assets/avoid_the_bomb/images/bomb.png")
                                .call()
                                .into_widget(),
                            ),
                            GameCellState::Flagged => Some(
                              ImageButton {
                                is_enabled: state.game_state == GameState::Playing,
                                file_path: "assets/avoid_the_bomb/images/flag.png",
                                on_right_mouse_up: Arc::new(AtomicRefCell::new(move || {
                                  let mut state = state_arc_1.write().unwrap();

                                  if state.is_done() {
                                    return;
                                  }

                                  if let Some(audio_tx) = context::MAIN_AUDIO_TX.get() {
                                    let _ = audio_tx.send(AudioReq::PlaySound(
                                      "assets/avoid_the_bomb/audio/click.wav",
                                    ));
                                  }

                                  state.set_cell_state(index, GameCellState::Hidden);
                                  state.flagged_bomb_count -= 1;
                                })),
                                ..Default::default()
                              }
                              .into_widget(),
                            ),
                          }
                        }
                      }
                    }),
                  }
                  .into_widget(),
                ])
                .call()
                .into_widget(),
            }
            .into_widget(),
          ),
          match state.game_state {
            GameState::Playing => None,
            GameState::Pause => Some(
              BackConfirmDialog {
                navigator,
                on_close: Arc::new(AtomicRefCell::new(move || {
                  // Resume the game
                  let mut state = state_arc_2.write().unwrap();
                  state.set_game_state(GameState::Playing);
                })),
              }
              .into_widget(),
            ),
            GameState::Dead => Some(
              GameOverDialog {
                navigator,
                on_ok: Arc::new(AtomicRefCell::new(move || {
                  // Restart the game
                  let mut state = state_arc_2.write().unwrap();
                  state.init();
                })),
              }
              .into_widget(),
            ),
            GameState::Won => Some(
              YouWonDialog {
                navigator,
                on_ok: Arc::new(AtomicRefCell::new(move || {
                  // Restart the game
                  let mut state = state_arc_2.write().unwrap();
                  state.init();
                })),
              }
              .into_widget(),
            ),
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
