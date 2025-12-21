use crate::{GameCell, GameCellType, consts, models::DirectedEdges, utils};
use fastrand::Rng;
use flut::{Context, models::Rect};
use indexmap::IndexSet;
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::collections::VecDeque;

// Generation settings
const WALL_MARGIN: u16 = 2;
const WORM_MARGIN: u16 = 8;
const WALL_COUNT: u16 = 20;

pub(super) struct Level {
  grid_cells: Box<[GameCell]>,
  air_positions: IndexSet<u16>,
  worm_positions: VecDeque<u16>, // Front is head, back is tail
}

impl Level {
  pub(super) fn new(context: &mut Context<'_>) -> Self {
    // Generate a land surrounded by walls with a worm in the middle
    let (cell_tys, rects): (Vec<_>, Vec<_>) = (0..consts::TOTAL_GRID_CELL_COUNT)
      .into_par_iter()
      .with_min_len(consts::MIN_SEQ_LEN)
      .map(|index| {
        let cell_ty = if index < consts::GRID_CELL_COUNTS.0 // Top wall
          || index % consts::GRID_CELL_COUNTS.0 == 0 // Left wall
          || index % consts::GRID_CELL_COUNTS.0 == consts::GRID_CELL_COUNTS.0 - 1 // Right wall
          || index >= (consts::GRID_CELL_COUNTS.1 - 1) * consts::GRID_CELL_COUNTS.0
        // Bottom wall
        {
          GameCellType::Wall // Wall
        } else if index == consts::TOTAL_GRID_CELL_COUNT >> 1 {
          GameCellType::Worm // Worm
        } else {
          GameCellType::Air // Air
        };

        (
          cell_ty,
          Rect {
            position: utils::to_position(index),
            size: consts::GRID_CELL_SIZE,
            color: cell_ty.get_color(),
          },
        )
      })
      .unzip();

    let grid_render_ids = context.renderer.bulk_add_rects(rects.into_boxed_slice());

    let mut grid_cells = cell_tys
      .par_iter()
      .with_min_len(consts::MIN_SEQ_LEN)
      .zip(grid_render_ids.par_iter().with_min_len(consts::MIN_SEQ_LEN))
      .map(|(&cell_ty, &render_id)| GameCell {
        ty: cell_ty,
        render_id,
      })
      .collect::<Box<_>>();

    let mut air_positions = cell_tys
      .into_par_iter()
      .with_min_len(consts::MIN_SEQ_LEN)
      .enumerate()
      .filter_map(|(index, cell_ty)| {
        if let GameCellType::Air = cell_ty {
          Some(index as _)
        } else {
          None
        }
      })
      .collect::<IndexSet<_>>();

    let mut final_air_positions = air_positions.clone();
    let worm_positions = VecDeque::from_iter([(consts::TOTAL_GRID_CELL_COUNT >> 1)]);
    let mut rng = Rng::new();
    alloc_top_wall_margin(&mut air_positions);
    alloc_left_wall_margin(&mut air_positions);
    alloc_right_wall_margin(&mut air_positions);
    alloc_bottom_wall_margin(&mut air_positions);
    alloc_worm_margin(&mut air_positions);

    // Prepare data structures to support efficient connected wall placements
    let mut air_positions_before_spawn_walls = air_positions.clone();

    // air_positions_after_spawn_walls
    let mut wall_positions = Vec::with_capacity(WALL_COUNT as _);

    let mut y_to_wall_positions = FxHashMap::with_capacity_and_hasher(
      WALL_COUNT.min(consts::GRID_CELL_COUNTS.1 - 2) as _,
      FxBuildHasher,
    );

    let mut x_to_wall_positions = FxHashMap::with_capacity_and_hasher(
      WALL_COUNT.min(consts::GRID_CELL_COUNTS.0 - 2) as _,
      FxBuildHasher,
    );

    // Spawn walls at random places
    for _ in 0..WALL_COUNT {
      let Some(&rand_air_position) = rng.choice(&air_positions) else {
        break;
      };

      alloc_wall_margin(&mut air_positions, rand_air_position);

      // Index rand_air_position to support efficient connected wall placements
      let rand_air_2d_position = utils::to_2d_index(rand_air_position);
      wall_positions.push(rand_air_position);

      y_to_wall_positions
        .entry(rand_air_2d_position.1)
        .or_insert_with(|| {
          Vec::with_capacity(((consts::GRID_CELL_COUNTS.0 - 2) / (WALL_MARGIN + 1)) as _)
        })
        .push(rand_air_position);

      x_to_wall_positions
        .entry(rand_air_2d_position.0)
        .or_insert_with(|| {
          Vec::with_capacity(((consts::GRID_CELL_COUNTS.1 - 2) / (WALL_MARGIN + 1)) as _)
        })
        .push(rand_air_position);
    }

    // Prepare data structure to support skip connected walls
    let mut connected_wall_positions =
      FxHashSet::with_capacity_and_hasher(WALL_COUNT as _, FxBuildHasher);

    // Connect some walls
    for wall_position in wall_positions {
      // Skip already connected walls
      if connected_wall_positions.contains(&wall_position) {
        continue;
      }

      let wall_2d_position = utils::to_2d_index(wall_position);
      let y_wall_positions = &y_to_wall_positions[&wall_2d_position.1];
      let x_wall_positions = &x_to_wall_positions[&wall_2d_position.0];

      // Prepare data structures to support efficient random sampling of walls to connect
      let mut up_wall_positions = IndexSet::with_capacity(x_wall_positions.len());
      let mut down_wall_positions = IndexSet::with_capacity(x_wall_positions.len());
      let mut left_wall_positions = IndexSet::with_capacity(y_wall_positions.len());
      let mut right_wall_positions = IndexSet::with_capacity(y_wall_positions.len());

      // Index the above data structures to support efficient random sampling of walls to connect
      for &y_wall_position in y_wall_positions {
        // Skip already connected walls
        if connected_wall_positions.contains(&y_wall_position) {
          continue;
        }

        let y_wall_2d_position = utils::to_2d_index(y_wall_position);
        let x_diff = y_wall_2d_position.0 as i32 - wall_2d_position.0 as i32;

        if x_diff < 0 {
          left_wall_positions.insert(y_wall_position);
        } else if x_diff > 0 {
          right_wall_positions.insert(y_wall_position);
        }
      }

      for &x_wall_position in x_wall_positions {
        // Skip already connected walls
        if connected_wall_positions.contains(&x_wall_position) {
          continue;
        }

        let x_wall_2d_position = utils::to_2d_index(x_wall_position);
        let y_diff = x_wall_2d_position.1 as i32 - wall_2d_position.1 as i32;

        if y_diff < 0 {
          up_wall_positions.insert(x_wall_position);
        } else if y_diff > 0 {
          down_wall_positions.insert(x_wall_position);
        }
      }

      // Select random walls to connect
      let rand_up_wall_position = rng.choice(&up_wall_positions);
      let rand_right_wall_position = rng.choice(&right_wall_positions);
      let rand_down_wall_position = rng.choice(&down_wall_positions);
      let rand_left_wall_position = rng.choice(&left_wall_positions);

      // Select which random walls to connect
      let directed_edges_candidates = DirectedEdges::get_candidates(
        rand_up_wall_position.is_some(),
        rand_right_wall_position.is_some(),
        rand_down_wall_position.is_some(),
        rand_left_wall_position.is_some(),
      );

      let rand_directed_edges = rng.choice(directed_edges_candidates).unwrap();

      match rand_directed_edges {
        DirectedEdges::Neutral => {
          connect_neutral_wall(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            wall_position,
          );

          alloc_neutral_wall_margin(&mut air_positions_before_spawn_walls, wall_position);
        }
        DirectedEdges::Up => {
          let rand_up_wall_position = *rand_up_wall_position.unwrap();

          connect_up_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &up_wall_positions,
            wall_position,
            rand_up_wall_position,
          );

          alloc_up_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_up_wall_position,
          );
        }
        DirectedEdges::Right => {
          let rand_right_wall_position = *rand_right_wall_position.unwrap();

          connect_right_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &right_wall_positions,
            wall_position,
            rand_right_wall_position,
          );

          alloc_right_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_right_wall_position,
          );
        }
        DirectedEdges::Down => {
          let rand_down_wall_position = *rand_down_wall_position.unwrap();

          connect_down_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &down_wall_positions,
            wall_position,
            rand_down_wall_position,
          );

          alloc_down_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_down_wall_position,
          );
        }
        DirectedEdges::Left => {
          let rand_left_wall_position = *rand_left_wall_position.unwrap();

          connect_left_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &left_wall_positions,
            wall_position,
            rand_left_wall_position,
          );

          alloc_left_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_left_wall_position,
          );
        }
        DirectedEdges::UpRight => {
          let rand_up_wall_position = *rand_up_wall_position.unwrap();
          let rand_right_wall_position = *rand_right_wall_position.unwrap();

          connect_up_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &up_wall_positions,
            wall_position,
            rand_up_wall_position,
          );

          connect_right_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &right_wall_positions,
            wall_position,
            rand_right_wall_position,
          );

          alloc_up_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_up_wall_position,
          );

          alloc_right_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_right_wall_position,
          );
        }
        DirectedEdges::RightDown => {
          let rand_right_wall_position = *rand_right_wall_position.unwrap();
          let rand_down_wall_position = *rand_down_wall_position.unwrap();

          connect_right_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &right_wall_positions,
            wall_position,
            rand_right_wall_position,
          );

          connect_down_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &down_wall_positions,
            wall_position,
            rand_down_wall_position,
          );

          alloc_right_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_right_wall_position,
          );

          alloc_down_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_down_wall_position,
          );
        }
        DirectedEdges::DownLeft => {
          let rand_down_wall_position = *rand_down_wall_position.unwrap();
          let rand_left_wall_position = *rand_left_wall_position.unwrap();

          connect_down_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &down_wall_positions,
            wall_position,
            rand_down_wall_position,
          );

          connect_left_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &left_wall_positions,
            wall_position,
            rand_left_wall_position,
          );

          alloc_down_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_down_wall_position,
          );

          alloc_left_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_left_wall_position,
          );
        }
        DirectedEdges::LeftUp => {
          let rand_left_wall_position = *rand_left_wall_position.unwrap();
          let rand_up_wall_position = *rand_up_wall_position.unwrap();

          connect_left_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &left_wall_positions,
            wall_position,
            rand_left_wall_position,
          );

          connect_up_walls(
            context,
            &mut grid_cells,
            &mut air_positions_before_spawn_walls,
            &mut final_air_positions,
            &mut connected_wall_positions,
            &up_wall_positions,
            wall_position,
            rand_up_wall_position,
          );

          alloc_left_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_left_wall_position,
          );

          alloc_up_wall_margins(
            &mut air_positions_before_spawn_walls,
            wall_position,
            rand_up_wall_position,
          );
        }
      }
    }

    Self {
      grid_cells,
      air_positions: final_air_positions,
      worm_positions,
    }
  }

  #[inline]
  pub(super) const fn get_grid_cell(&self, position: u16) -> GameCellType {
    self.grid_cells[position as usize].ty
  }

  pub(super) fn set_grid_cell(
    &mut self,
    context: &mut Context<'_>,
    position: u16,
    cell_ty: GameCellType,
  ) {
    let grid_cell = &mut self.grid_cells[position as usize];
    grid_cell.ty = cell_ty;

    context.renderer.update_rect(
      grid_cell.render_id,
      Rect {
        position: utils::to_position(position),
        size: consts::GRID_CELL_SIZE,
        color: cell_ty.get_color(),
      },
    );
  }

  #[inline]
  pub(super) const fn get_worm_positions(&self) -> &VecDeque<u16> {
    &self.worm_positions
  }

  #[inline]
  pub(super) const fn get_worm_positions_mut(&mut self) -> &mut VecDeque<u16> {
    &mut self.worm_positions
  }

  #[inline]
  pub(super) const fn get_air_positions(&self) -> &IndexSet<u16> {
    &self.air_positions
  }

  #[inline]
  pub(super) const fn get_air_positions_mut(&mut self) -> &mut IndexSet<u16> {
    &mut self.air_positions
  }
}

fn alloc_top_wall_margin(air_positions: &mut IndexSet<u16>) {
  for y in 1..=WALL_MARGIN {
    for x in 1..consts::GRID_CELL_COUNTS.0 - 1 {
      air_positions.swap_remove(&(y * consts::GRID_CELL_COUNTS.0 + x));
    }
  }
}

fn alloc_left_wall_margin(air_positions: &mut IndexSet<u16>) {
  for x in 1..=WALL_MARGIN {
    for y in 1..consts::GRID_CELL_COUNTS.1 - 1 {
      air_positions.swap_remove(&(y * consts::GRID_CELL_COUNTS.0 + x));
    }
  }
}

fn alloc_right_wall_margin(air_positions: &mut IndexSet<u16>) {
  for x in consts::GRID_CELL_COUNTS.0 - 1 - WALL_MARGIN..consts::GRID_CELL_COUNTS.0 - 1 {
    for y in 1..consts::GRID_CELL_COUNTS.1 - 1 {
      air_positions.swap_remove(&(y * consts::GRID_CELL_COUNTS.0 + x));
    }
  }
}

fn alloc_bottom_wall_margin(air_positions: &mut IndexSet<u16>) {
  for y in consts::GRID_CELL_COUNTS.1 - 1 - WALL_MARGIN..consts::GRID_CELL_COUNTS.1 - 1 {
    for x in 1..consts::GRID_CELL_COUNTS.0 - 1 {
      air_positions.swap_remove(&(y * consts::GRID_CELL_COUNTS.0 + x));
    }
  }
}

fn alloc_worm_margin(air_positions: &mut IndexSet<u16>) {
  for y_offset in -(WORM_MARGIN as i32)..=(WORM_MARGIN as i32) {
    for x_offset in -(WORM_MARGIN as i32)..=(WORM_MARGIN as i32) {
      air_positions.swap_remove(
        &(((consts::TOTAL_GRID_CELL_COUNT >> 1) as i32
          + y_offset * consts::GRID_CELL_COUNTS.0 as i32
          + x_offset) as u16),
      );
    }
  }
}

fn alloc_wall_margin(air_positions: &mut IndexSet<u16>, wall_position: u16) {
  for y_offset in -(WALL_MARGIN as i32)..=(WALL_MARGIN as i32) {
    for x_offset in -(WALL_MARGIN as i32)..=(WALL_MARGIN as i32) {
      air_positions.swap_remove(
        &((wall_position as i32 + y_offset * consts::GRID_CELL_COUNTS.0 as i32 + x_offset) as u16),
      );
    }
  }
}

fn connect_neutral_wall(
  context: &mut Context<'_>,
  grid_cells: &mut [GameCell],
  air_positions: &mut IndexSet<u16>,
  final_air_positions: &mut IndexSet<u16>,
  connected_wall_positions: &mut FxHashSet<u16>,
  wall_position: u16,
) {
  if !air_positions.contains(&wall_position) {
    return;
  }

  let grid_cell = &mut grid_cells[wall_position as usize];
  grid_cell.ty = GameCellType::Wall;
  final_air_positions.swap_remove(&wall_position);

  context.renderer.update_rect(
    grid_cell.render_id,
    Rect {
      position: utils::to_position(wall_position),
      size: consts::GRID_CELL_SIZE,
      color: GameCellType::Wall.get_color(),
    },
  );

  connected_wall_positions.insert(wall_position);
}

fn alloc_neutral_wall_margin(air_positions: &mut IndexSet<u16>, wall_position: u16) {
  alloc_wall_margin(air_positions, wall_position);
}

fn connect_up_walls(
  context: &mut Context<'_>,
  grid_cells: &mut [GameCell],
  air_positions: &mut IndexSet<u16>,
  final_air_positions: &mut IndexSet<u16>,
  connected_wall_positions: &mut FxHashSet<u16>,
  up_wall_positions: &IndexSet<u16>,
  src_wall_position: u16,
  dst_wall_position: u16,
) {
  let src_wall_2d_position = utils::to_2d_index(src_wall_position);
  let dst_wall_2d_position = utils::to_2d_index(dst_wall_position);

  for y in dst_wall_2d_position.1..=src_wall_2d_position.1 {
    let wall_position_to_connect = y * consts::GRID_CELL_COUNTS.0 + src_wall_2d_position.0;

    if up_wall_positions.contains(&wall_position_to_connect) {
      connected_wall_positions.insert(wall_position_to_connect);
    }

    if !air_positions.contains(&wall_position_to_connect) {
      continue;
    }

    let grid_cell = &mut grid_cells[wall_position_to_connect as usize];
    grid_cell.ty = GameCellType::Wall;
    final_air_positions.swap_remove(&wall_position_to_connect);

    context.renderer.update_rect(
      grid_cell.render_id,
      Rect {
        position: utils::to_position(wall_position_to_connect),
        size: consts::GRID_CELL_SIZE,
        color: GameCellType::Wall.get_color(),
      },
    );
  }

  connected_wall_positions.extend([src_wall_position, dst_wall_position]);
}

fn alloc_up_wall_margins(
  air_positions: &mut IndexSet<u16>,
  src_wall_position: u16,
  dst_wall_position: u16,
) {
  let src_wall_2d_position = utils::to_2d_index(src_wall_position);
  let dst_wall_2d_position = utils::to_2d_index(dst_wall_position);

  for y in dst_wall_2d_position.1..=src_wall_2d_position.1 {
    alloc_wall_margin(
      air_positions,
      y * consts::GRID_CELL_COUNTS.0 + src_wall_2d_position.0,
    );
  }
}

fn connect_right_walls(
  context: &mut Context<'_>,
  grid_cells: &mut [GameCell],
  air_positions: &mut IndexSet<u16>,
  final_air_positions: &mut IndexSet<u16>,
  connected_wall_positions: &mut FxHashSet<u16>,
  right_wall_positions: &IndexSet<u16>,
  src_wall_position: u16,
  dst_wall_position: u16,
) {
  let src_wall_2d_position = utils::to_2d_index(src_wall_position);
  let dst_wall_2d_position = utils::to_2d_index(dst_wall_position);

  for x in src_wall_2d_position.0..=dst_wall_2d_position.0 {
    let wall_position_to_connect = src_wall_2d_position.1 * consts::GRID_CELL_COUNTS.0 + x;

    if right_wall_positions.contains(&wall_position_to_connect) {
      connected_wall_positions.insert(wall_position_to_connect);
    }

    if !air_positions.contains(&wall_position_to_connect) {
      continue;
    }

    let grid_cell = &mut grid_cells[wall_position_to_connect as usize];
    grid_cell.ty = GameCellType::Wall;
    final_air_positions.swap_remove(&wall_position_to_connect);

    context.renderer.update_rect(
      grid_cell.render_id,
      Rect {
        position: utils::to_position(wall_position_to_connect),
        size: consts::GRID_CELL_SIZE,
        color: GameCellType::Wall.get_color(),
      },
    );
  }

  connected_wall_positions.extend([src_wall_position, dst_wall_position]);
}

fn alloc_right_wall_margins(
  air_positions: &mut IndexSet<u16>,
  src_wall_position: u16,
  dst_wall_position: u16,
) {
  let src_wall_2d_position = utils::to_2d_index(src_wall_position);
  let dst_wall_2d_position = utils::to_2d_index(dst_wall_position);

  for x in src_wall_2d_position.0..=dst_wall_2d_position.0 {
    alloc_wall_margin(
      air_positions,
      src_wall_2d_position.1 * consts::GRID_CELL_COUNTS.0 + x,
    );
  }
}

fn connect_down_walls(
  context: &mut Context<'_>,
  grid_cells: &mut [GameCell],
  air_positions: &mut IndexSet<u16>,
  final_air_positions: &mut IndexSet<u16>,
  connected_wall_positions: &mut FxHashSet<u16>,
  down_wall_positions: &IndexSet<u16>,
  src_wall_position: u16,
  dst_wall_position: u16,
) {
  let src_wall_2d_position = utils::to_2d_index(src_wall_position);
  let dst_wall_2d_position = utils::to_2d_index(dst_wall_position);

  for y in src_wall_2d_position.1..=dst_wall_2d_position.1 {
    let wall_position_to_connect = y * consts::GRID_CELL_COUNTS.0 + src_wall_2d_position.0;

    if down_wall_positions.contains(&wall_position_to_connect) {
      connected_wall_positions.insert(wall_position_to_connect);
    }

    if !air_positions.contains(&wall_position_to_connect) {
      continue;
    }

    let grid_cell = &mut grid_cells[wall_position_to_connect as usize];
    grid_cell.ty = GameCellType::Wall;
    final_air_positions.swap_remove(&wall_position_to_connect);

    context.renderer.update_rect(
      grid_cell.render_id,
      Rect {
        position: utils::to_position(wall_position_to_connect),
        size: consts::GRID_CELL_SIZE,
        color: GameCellType::Wall.get_color(),
      },
    );
  }

  connected_wall_positions.extend([src_wall_position, dst_wall_position]);
}

fn alloc_down_wall_margins(
  air_positions: &mut IndexSet<u16>,
  src_wall_position: u16,
  dst_wall_position: u16,
) {
  let src_wall_2d_position = utils::to_2d_index(src_wall_position);
  let dst_wall_2d_position = utils::to_2d_index(dst_wall_position);

  for y in src_wall_2d_position.1..=dst_wall_2d_position.1 {
    alloc_wall_margin(
      air_positions,
      y * consts::GRID_CELL_COUNTS.0 + src_wall_2d_position.0,
    );
  }
}

fn connect_left_walls(
  context: &mut Context<'_>,
  grid_cells: &mut [GameCell],
  air_positions: &mut IndexSet<u16>,
  final_air_positions: &mut IndexSet<u16>,
  connected_wall_positions: &mut FxHashSet<u16>,
  left_wall_positions: &IndexSet<u16>,
  src_wall_position: u16,
  dst_wall_position: u16,
) {
  let src_wall_2d_position = utils::to_2d_index(src_wall_position);
  let dst_wall_2d_position = utils::to_2d_index(dst_wall_position);

  for x in dst_wall_2d_position.0..=src_wall_2d_position.0 {
    let wall_position_to_connect = src_wall_2d_position.1 * consts::GRID_CELL_COUNTS.0 + x;

    if left_wall_positions.contains(&wall_position_to_connect) {
      connected_wall_positions.insert(wall_position_to_connect);
    }

    if !air_positions.contains(&wall_position_to_connect) {
      continue;
    }

    let grid_cell = &mut grid_cells[wall_position_to_connect as usize];
    grid_cell.ty = GameCellType::Wall;
    final_air_positions.swap_remove(&wall_position_to_connect);

    context.renderer.update_rect(
      grid_cell.render_id,
      Rect {
        position: utils::to_position(wall_position_to_connect),
        size: consts::GRID_CELL_SIZE,
        color: GameCellType::Wall.get_color(),
      },
    );
  }

  connected_wall_positions.extend([src_wall_position, dst_wall_position]);
}

fn alloc_left_wall_margins(
  air_positions: &mut IndexSet<u16>,
  src_wall_position: u16,
  dst_wall_position: u16,
) {
  let src_wall_2d_position = utils::to_2d_index(src_wall_position);
  let dst_wall_2d_position = utils::to_2d_index(dst_wall_position);

  for x in dst_wall_2d_position.0..=src_wall_2d_position.0 {
    alloc_wall_margin(
      air_positions,
      src_wall_2d_position.1 * consts::GRID_CELL_COUNTS.0 + x,
    );
  }
}
