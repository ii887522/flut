#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

mod models;

use crate::models::{Direction, GameCell, GameCellType};
use flut::{Context, Event, Keycode, models::Rect};
use indexmap::{IndexSet, indexset};
use kira::{
  Tween,
  sound::{FromFileError, streaming::StreamingSoundHandle},
};
use rayon::prelude::*;
use std::collections::VecDeque;

// General settings
const MIN_SEQ_LEN: usize = 256;
pub const WINDOW_SIZE: (u32, u32) = (1280, 720);
const UPDATES_PER_SECOND: f32 = 30.0;

// Shake settings
const SHAKE_DURATION: f32 = 0.5;
const SHAKE_STRENGTH: f32 = 64.0;

// Grid settings
const GRID_SIZE: (f32, f32) = (684.0, 684.0);
const GRID_CELL_SIZE: (f32, f32) = (12.0, 12.0);
const GRID_CELL_MARGIN: (f32, f32) = (2.0, 2.0);

// Computed grid settings
const GRID_POSITION: (f32, f32) = (
  (WINDOW_SIZE.0 as f32 - GRID_SIZE.0) * 0.5,
  (WINDOW_SIZE.1 as f32 - GRID_SIZE.1) * 0.5,
);
const GRID_CELL_COUNTS: (u32, u32) = (
  ((GRID_SIZE.0 + GRID_CELL_MARGIN.0) / (GRID_CELL_SIZE.0 + GRID_CELL_MARGIN.0)) as _,
  ((GRID_SIZE.1 + GRID_CELL_MARGIN.1) / (GRID_CELL_SIZE.1 + GRID_CELL_MARGIN.1)) as _,
);
const TOTAL_GRID_CELL_COUNT: u32 = GRID_CELL_COUNTS.0 * GRID_CELL_COUNTS.1;

pub struct Game {
  grid_cells: Box<[GameCell]>,
  air_positions: IndexSet<u16>,
  worm_positions: VecDeque<u16>, // Front is head, back is tail
  worm_direction: Direction,
  input_worm_direction: Option<Direction>,
  worm_move_music: Option<StreamingSoundHandle<FromFileError>>,
  worm_dead: bool,
  accum: f32,
  shake_accum: f32,
}

impl Default for Game {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl Game {
  #[inline]
  pub fn new() -> Self {
    Self {
      grid_cells: Box::new([]),
      air_positions: indexset! {},
      worm_positions: VecDeque::from_iter([(TOTAL_GRID_CELL_COUNT >> 1) as _]),
      worm_direction: Direction::rand(),
      input_worm_direction: None,
      worm_move_music: None,
      worm_dead: false,
      accum: 0.0,
      shake_accum: 0.0,
    }
  }

  fn set_grid_cell(&mut self, context: &mut Context<'_>, position: u16, cell_ty: GameCellType) {
    let grid_cell = &mut self.grid_cells[position as usize];
    grid_cell.ty = cell_ty;

    context.renderer.update_rect(
      grid_cell.render_id,
      Rect {
        position: to_position(position as _),
        size: GRID_CELL_SIZE,
        color: cell_ty.get_color(),
      },
    );
  }

  fn move_worm(&mut self, context: &mut Context<'_>, new_worm_head_position: u16) {
    let worm_tail_position = self.worm_positions.pop_back().unwrap();
    self.worm_positions.push_front(new_worm_head_position);
    self.air_positions.swap_remove(&new_worm_head_position);
    self.air_positions.insert(worm_tail_position);
    self.set_grid_cell(context, worm_tail_position, GameCellType::Air);
    self.set_grid_cell(context, new_worm_head_position, GameCellType::Worm);
  }

  fn grow_worm(&mut self, context: &mut Context<'_>, new_worm_head_position: u16) {
    if let Some(audio_manager) = context.audio_manager {
      audio_manager.play_sound("assets/worm/audios/eat.mp3");
    }

    self.worm_positions.push_front(new_worm_head_position);
    self.set_grid_cell(context, new_worm_head_position, GameCellType::Worm);
  }

  fn spawn_food(&mut self, context: &mut Context<'_>) {
    let air_position_to_remove = self
      .air_positions
      .swap_remove_index(fastrand::usize(..self.air_positions.len()))
      .unwrap();

    self.set_grid_cell(context, air_position_to_remove, GameCellType::Food);
  }

  fn kill_worm(&mut self, context: &mut Context<'_>) {
    if let Some(audio_manager) = context.audio_manager {
      audio_manager.play_sound("assets/worm/audios/hit.wav");
    }

    if let Some(mut worm_move_music) = self.worm_move_music.take() {
      worm_move_music.stop(Tween::default());
    }

    self.worm_dead = true;
  }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn init(game: &mut Game, mut context: Context<'_>) {
  if let Some(audio_manager) = context.audio_manager
    && let Some(mut worm_move_music) = audio_manager.play_music("assets/worm/audios/worm_move.mp3")
  {
    worm_move_music.set_loop_region(0.2..);
    worm_move_music.set_volume(-10.0, Tween::default());
    game.worm_move_music = Some(worm_move_music);
  }

  let (cell_tys, grid_rects): (Vec<_>, Vec<_>) = (0..TOTAL_GRID_CELL_COUNT)
    .into_par_iter()
    .with_min_len(MIN_SEQ_LEN)
    .map(|index| {
      let cell_ty = if index < GRID_CELL_COUNTS.0 // Top wall
          || index % GRID_CELL_COUNTS.0 == 0 // Left wall
          || index % GRID_CELL_COUNTS.0 == GRID_CELL_COUNTS.0 - 1 // Right wall
          || index >= (GRID_CELL_COUNTS.1 - 1) * GRID_CELL_COUNTS.0
      // Bottom wall
      {
        GameCellType::Wall // Wall
      } else if index == TOTAL_GRID_CELL_COUNT >> 1 {
        GameCellType::Worm // Worm
      } else {
        GameCellType::Air // Air
      };

      (
        cell_ty,
        Rect {
          position: to_position(index),
          size: GRID_CELL_SIZE,
          color: cell_ty.get_color(),
        },
      )
    })
    .unzip();

  let grid_render_ids = context
    .renderer
    .bulk_add_rects(grid_rects.into_boxed_slice());

  let grid_cells = cell_tys
    .par_iter()
    .with_min_len(MIN_SEQ_LEN)
    .zip(grid_render_ids.par_iter().with_min_len(MIN_SEQ_LEN))
    .map(|(&cell_ty, &render_id)| GameCell {
      ty: cell_ty,
      render_id,
    })
    .collect::<Box<_>>();

  let air_positions = cell_tys
    .into_par_iter()
    .with_min_len(MIN_SEQ_LEN)
    .enumerate()
    .filter_map(|(index, cell_ty)| {
      if let GameCellType::Air = cell_ty {
        Some(index as _)
      } else {
        None
      }
    })
    .collect::<IndexSet<_>>();

  game.grid_cells = grid_cells;
  game.air_positions = air_positions;
  game.spawn_food(&mut context);
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn process_event(game: &mut Game, event: Event) {
  match event {
    Event::KeyDown {
      keycode: Some(Keycode::W | Keycode::Up),
      ..
    } if game.worm_direction != Direction::Down => game.input_worm_direction = Some(Direction::Up),
    Event::KeyDown {
      keycode: Some(Keycode::D | Keycode::Right),
      ..
    } if game.worm_direction != Direction::Left => {
      game.input_worm_direction = Some(Direction::Right)
    }
    Event::KeyDown {
      keycode: Some(Keycode::S | Keycode::Down),
      ..
    } if game.worm_direction != Direction::Up => game.input_worm_direction = Some(Direction::Down),
    Event::KeyDown {
      keycode: Some(Keycode::A | Keycode::Left),
      ..
    } if game.worm_direction != Direction::Right => {
      game.input_worm_direction = Some(Direction::Left)
    }
    _ => {}
  }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn update(game: &mut Game, dt: f32, mut context: Context<'_>) {
  game.accum += dt;

  if game.worm_dead {
    game.shake_accum += dt;
  }

  if game.accum < 1.0 / UPDATES_PER_SECOND {
    return;
  }

  game.accum -= 1.0 / UPDATES_PER_SECOND;

  if game.worm_dead {
    if game.shake_accum < SHAKE_DURATION {
      context.renderer.set_cam_position(Some((
        fastrand::f32() * SHAKE_STRENGTH,
        fastrand::f32() * SHAKE_STRENGTH,
      )));
    } else {
      game.shake_accum = SHAKE_DURATION;
      context.renderer.set_cam_position(None);
    }

    return;
  }

  if let Some(input_worm_direction) = game.input_worm_direction.take() {
    game.worm_direction = input_worm_direction;
  }

  let worm_head_position = *game.worm_positions.front().unwrap();

  let new_worm_head_position = match game.worm_direction {
    Direction::Up => worm_head_position - GRID_CELL_COUNTS.0 as u16,
    Direction::Right => worm_head_position + 1,
    Direction::Down => worm_head_position + GRID_CELL_COUNTS.0 as u16,
    Direction::Left => worm_head_position - 1,
  };

  match game.grid_cells[new_worm_head_position as usize].ty {
    GameCellType::Air => game.move_worm(&mut context, new_worm_head_position),
    GameCellType::Wall | GameCellType::Worm => game.kill_worm(&mut context),
    GameCellType::Food => {
      game.grow_worm(&mut context, new_worm_head_position);
      game.spawn_food(&mut context);
    }
  }
}

#[inline]
const fn to_position(index: u32) -> (f32, f32) {
  (
    (index % GRID_CELL_COUNTS.0) as f32 * (GRID_CELL_SIZE.0 + GRID_CELL_MARGIN.0) + GRID_POSITION.0,
    (index / GRID_CELL_COUNTS.0) as f32 * (GRID_CELL_SIZE.1 + GRID_CELL_MARGIN.1) + GRID_POSITION.1,
  )
}
