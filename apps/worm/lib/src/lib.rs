#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

mod models;

use crate::models::{Air, WormCell};
use flut::{Event, models::Rect, renderers::RendererRef};
use rayon::prelude::*;

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

pub struct Game {
  airs: Vec<Air>,
  worm_cells: Vec<WormCell>, // Front is head, back is tail
}

impl Game {
  #[inline]
  pub const fn new() -> Self {
    Self {
      airs: vec![],
      worm_cells: vec![],
    }
  }
}

impl Default for Game {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn init(game: &mut Game, mut renderer: RendererRef<'_>) {
  let (is_airs, grid_cell_rects): (Vec<_>, Vec<_>) = (0..TOTAL_GRID_CELL_COUNT)
    .into_par_iter()
    .with_min_len(MIN_SEQ_LEN)
    .map(|index| {
      let (is_air, color) = if index < GRID_CELL_COUNTS.0 // Top wall
          || index % GRID_CELL_COUNTS.0 == 0 // Left wall
          || index % GRID_CELL_COUNTS.0 == GRID_CELL_COUNTS.0 - 1 // Right wall
          || index >= (GRID_CELL_COUNTS.1 - 1) * GRID_CELL_COUNTS.0
      // Bottom wall
      {
        // Wall
        (false, (1.0, 0.0, 0.0, 1.0))
      } else if index == TOTAL_GRID_CELL_COUNT >> 1 {
        // Worm
        (false, (0.918, 0.663, 0.596, 1.0))
      } else {
        // Air
        (true, (0.15, 0.15, 0.15, 1.0))
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

  let grid_cell_render_ids = renderer.bulk_add_rects(grid_cell_rects.into_boxed_slice());

  let mut airs = is_airs
    .into_par_iter()
    .with_min_len(MIN_SEQ_LEN)
    .zip(grid_cell_render_ids.par_iter().with_min_len(MIN_SEQ_LEN))
    .enumerate()
    .filter_map(|(index, (is_air, &render_id))| {
      if is_air {
        Some(Air {
          position: index as _,
          render_id,
        })
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  let air_to_remove = airs.swap_remove(fastrand::usize(..airs.len()));

  renderer.update_rect(
    air_to_remove.render_id,
    Rect {
      position: to_position(air_to_remove.position as _),
      size: GRID_CELL_SIZE,
      color: (0.0, 1.0, 0.0, 1.0),
    },
  );

  let worm_cells = vec![WormCell {
    position: (TOTAL_GRID_CELL_COUNT >> 1) as _,
    render_id: grid_cell_render_ids[(TOTAL_GRID_CELL_COUNT >> 1) as usize],
  }];

  game.airs = airs;
  game.worm_cells = worm_cells;
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn process_event(_game: &mut Game, _event: Event) {
  // todo
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn update(_game: &mut Game, _dt: f32, _renderer: RendererRef<'_>) {
  // todo
}

#[inline]
const fn to_position(index: u32) -> (f32, f32) {
  (
    (index % GRID_CELL_COUNTS.0) as f32 * (GRID_CELL_SIZE.0 + GRID_CELL_MARGIN.0) + GRID_POSITION.0,
    (index / GRID_CELL_COUNTS.0) as f32 * (GRID_CELL_SIZE.1 + GRID_CELL_MARGIN.1) + GRID_POSITION.1,
  )
}
