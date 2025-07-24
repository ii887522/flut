#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod consts;
mod models;

use crate::models::{Air, Direction, Wall, Worm};
use flut::{App, Clock, Renderer, app, collections::SparseSet, models::Rect};
use mimalloc::MiMalloc;
use rayon::{iter::Either, prelude::*};
use sdl2::event::Event;
use std::collections::VecDeque;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
  app::run(WormGame::new());
}

struct WormGame {
  airs: SparseSet<Air>,
  walls: Box<[Wall]>,
  worms: VecDeque<Worm>, // Front is tail, back is head
  clock: Clock,
}

impl WormGame {
  fn new() -> Self {
    let ((walls, worms), airs): ((Vec<_>, Vec<_>), Vec<_>) = (0..consts::CELL_COUNT)
      .into_par_iter()
      .partition_map(|index| {
        if index < consts::GRID_SIZE.0 // Top wall
          || index % consts::GRID_SIZE.0 == 0 // Left wall
          || (index + 1) % consts::GRID_SIZE.0 == 0 // Right wall
          || index >= consts::GRID_SIZE.0 * (consts::GRID_SIZE.1 - 1)
        // Bottom wall
        {
          Either::Left(Either::Left(Wall {
            position: index,
            drawable_id: u32::MAX,
          }))
        } else if index == consts::INITIAL_WORM_POSITION {
          Either::Left(Either::Right(Worm {
            position: index,
            drawable_id: u32::MAX,
            direction: Direction::Up,
          }))
        } else {
          Either::Right((
            index,
            Air {
              position: index,
              drawable_id: u32::MAX,
            },
          ))
        }
      });

    Self {
      airs: SparseSet::from_par_iter(airs),
      walls: walls.into_boxed_slice(),
      worms: VecDeque::from_iter(worms),
      clock: Clock::new(1.0 / consts::UPDATES_PER_SECOND),
    }
  }
}

impl App for WormGame {
  fn get_config(&self) -> app::Config {
    app::Config {
      title: "Worm".into(),
      size: consts::APP_SIZE,
      favicon_path: "assets/worm/favicon.png".into(),
      ..Default::default()
    }
  }

  fn init(&mut self, renderer: &mut dyn Renderer) {
    let rect_ids = renderer.add_rects(
      self
        .walls
        .par_iter()
        .map(|&wall| Rect::from(wall))
        .chain(self.worms.par_iter().map(|&worm| Rect::from(worm)))
        .chain(self.airs.get_dense().par_iter().map(|&air| Rect::from(air)))
        .collect(),
    );

    let (wall_ids, rect_ids) = rect_ids.split_at(self.walls.len());
    let (worm_ids, air_ids) = rect_ids.split_at(self.worms.len());

    self
      .walls
      .par_iter_mut()
      .zip(wall_ids.par_iter())
      .for_each(|(wall, &id)| wall.drawable_id = id);

    self
      .worms
      .iter_mut()
      .zip(worm_ids.iter())
      .for_each(|(worm, &id)| worm.drawable_id = id);

    self
      .airs
      .get_dense_mut()
      .par_iter_mut()
      .zip(air_ids.par_iter())
      .for_each(|(air, &id)| air.drawable_id = id);
  }

  fn process_event(&mut self, _event: Event) {}

  fn update(&mut self, dt: f32, renderer: &mut dyn Renderer) {
    if !self.clock.update(dt) {
      return;
    }

    let worm_head = *self.worms.back().unwrap();
    let worm_head_next_position = worm_head.calc_next_position();

    let Some(remove_air_resp) = self.airs.remove(worm_head_next_position) else {
      return;
    };

    let air = remove_air_resp.item;
    let worm_tail = self.worms.pop_front().unwrap();

    let new_worm_head = Worm {
      position: worm_head_next_position,
      drawable_id: worm_tail.drawable_id,
      direction: worm_head.direction,
    };

    let new_air = Air {
      position: worm_tail.position,
      drawable_id: air.drawable_id,
    };

    self.worms.push_back(new_worm_head);
    self.airs.push_by_id(worm_tail.position, new_air);
    renderer.update_rect(worm_tail.drawable_id, Rect::from(new_worm_head));
    renderer.update_rect(air.drawable_id, Rect::from(new_air));
  }
}
