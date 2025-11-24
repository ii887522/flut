#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

mod models;

use crate::models::Direction;
use flut::{
  Event,
  models::Rect,
  renderers::{RendererRef, renderer_ref},
};
use rayon::prelude::*;
use std::collections::VecDeque;

// General settings
const MIN_SEQ_LEN: usize = 256;
pub const WINDOW_SIZE: (u32, u32) = (1600, 900);

// Grid settings
const GRID_SIZE: (f32, f32) = (880.0, 880.0);
const GRID_CELL_SIZE: (f32, f32) = (16.0, 16.0);
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

// Color settings
const AIR_COLOR: (f32, f32, f32, f32) = (0.15, 0.15, 0.15, 1.0);
const WALL_COLOR: (f32, f32, f32, f32) = (1.0, 0.0, 0.0, 1.0);
const WORM_COLOR: (f32, f32, f32, f32) = (0.918, 0.663, 0.596, 1.0);
const FOOD_COLOR: (f32, f32, f32, f32) = (0.0, 1.0, 0.0, 1.0);

pub struct Game {
  grid_render_ids: Box<[renderer_ref::Id]>,
  air_positions: Vec<u16>,
  worm_positions: VecDeque<u16>, // Front is head, back is tail
  worm_direction: Direction,
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
      grid_render_ids: Box::new([]),
      air_positions: vec![],
      worm_positions: VecDeque::new(),
      worm_direction: Direction::rand(),
    }
  }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn init(game: &mut Game, mut renderer: RendererRef<'_>) {
  let (is_airs, grid_rects): (Vec<_>, Vec<_>) = (0..TOTAL_GRID_CELL_COUNT)
    .into_par_iter()
    .with_min_len(MIN_SEQ_LEN)
    .map(|index| {
      let (is_air, color) = if index < GRID_CELL_COUNTS.0 // Top wall
          || index % GRID_CELL_COUNTS.0 == 0 // Left wall
          || index % GRID_CELL_COUNTS.0 == GRID_CELL_COUNTS.0 - 1 // Right wall
          || index >= (GRID_CELL_COUNTS.1 - 1) * GRID_CELL_COUNTS.0
      // Bottom wall
      {
        (false, WALL_COLOR) // Wall
      } else if index == TOTAL_GRID_CELL_COUNT >> 1 {
        (false, WORM_COLOR) // Worm
      } else {
        (true, AIR_COLOR) // Air
      };

      (
        is_air,
        Rect {
          position: to_position(index),
          size: GRID_CELL_SIZE,
          color,
        },
      )
    })
    .unzip();

  let grid_render_ids = renderer.bulk_add_rects(grid_rects.into_boxed_slice());

  let mut air_positions = is_airs
    .into_par_iter()
    .with_max_len(MIN_SEQ_LEN)
    .enumerate()
    .filter_map(|(index, is_air)| if is_air { Some(index as _) } else { None })
    .collect::<Vec<_>>();

  let air_position_to_remove = air_positions.swap_remove(fastrand::usize(..air_positions.len()));

  renderer.update_rect(
    grid_render_ids[air_position_to_remove as usize],
    Rect {
      position: to_position(air_position_to_remove as _),
      size: GRID_CELL_SIZE,
      color: FOOD_COLOR,
    },
  );

  let worm_positions = VecDeque::from_iter([(TOTAL_GRID_CELL_COUNT >> 1) as _]);

  game.grid_render_ids = grid_render_ids;
  game.air_positions = air_positions;
  game.worm_positions = worm_positions;
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn process_event(_game: &mut Game, _event: Event) {
  // todo
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn update(game: &mut Game, _dt: f32, mut renderer: RendererRef<'_>) {
  let worm_head_position = *game.worm_positions.front().unwrap();
  let worm_tail_position = game.worm_positions.pop_back().unwrap();

  let new_worm_head_position = match game.worm_direction {
    Direction::Up => worm_head_position - GRID_CELL_COUNTS.0 as u16,
    Direction::Right => worm_head_position + 1,
    Direction::Down => worm_head_position + GRID_CELL_COUNTS.0 as u16,
    Direction::Left => worm_head_position - 1,
  };

  game.worm_positions.push_front(new_worm_head_position);

  renderer.update_rect(
    game.grid_render_ids[worm_tail_position as usize],
    Rect {
      position: to_position(worm_tail_position as _),
      size: GRID_CELL_SIZE,
      color: AIR_COLOR,
    },
  );

  renderer.update_rect(
    game.grid_render_ids[new_worm_head_position as usize],
    Rect {
      position: to_position(new_worm_head_position as _),
      size: GRID_CELL_SIZE,
      color: WORM_COLOR,
    },
  );
}

#[inline]
const fn to_position(index: u32) -> (f32, f32) {
  (
    (index % GRID_CELL_COUNTS.0) as f32 * (GRID_CELL_SIZE.0 + GRID_CELL_MARGIN.0) + GRID_POSITION.0,
    (index / GRID_CELL_COUNTS.0) as f32 * (GRID_CELL_SIZE.1 + GRID_CELL_MARGIN.1) + GRID_POSITION.1,
  )
}
