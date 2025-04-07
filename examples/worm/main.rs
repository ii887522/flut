#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod consts;
mod models;

use flut::{App, AppConfig, Clock, Engine, app, collections::SparseVec};
use models::{Air, Direction, Food, Wall, Worm};
use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};
use std::sync::atomic::{AtomicU16, Ordering};

fn main() {
  app::run(WormApp::new());
}

struct WormApp {
  clock: Clock,
  air: SparseVec<Air>,
  worm: Worm,
  food: Food,
  input_worm_direction: Option<Direction>,
}

impl WormApp {
  fn new() -> Self {
    let clock = Clock::new(consts::UPDATES_PER_SECOND);
    let mut air = SparseVec::with_capacity((consts::GRID_SIZE.0 * consts::GRID_SIZE.1) as _);

    // Returned values are ignored since we assumed they represent air positions
    air.par_extend(
      (0..consts::GRID_SIZE.0 * consts::GRID_SIZE.1)
        .into_par_iter()
        .map(|index| Air {
          position: index,
          drawable_id: u16::MAX, // Will be initialized in WormApp::init() method
        })
        .collect(),
    );

    air.par_remove(
      &(0..consts::GRID_SIZE.0 * consts::GRID_SIZE.1)
        .into_par_iter()
        .filter(|&index| {
          // Allocated by worm
          index == consts::INITIAL_WORM_POSITION_1D ||
          // Allocated by top walls
          index < consts::GRID_SIZE.0 ||
          // Allocated by bottom walls
          index >= (consts::GRID_SIZE.1 - 1) * consts::GRID_SIZE.0 ||
          // Allocated by left walls
          index % consts::GRID_SIZE.0 == 0 ||
          // Allocated by right walls
          index % consts::GRID_SIZE.0 == consts::GRID_SIZE.0 - 1
        })
        .collect::<Vec<_>>(),
    );

    let worm = Worm {
      position: consts::INITIAL_WORM_POSITION_1D,
      direction: Direction::rand(),
      drawable_id: AtomicU16::new(u16::MAX), // Will be initialized in WormApp::init() method
    };

    let rand_air = air.remove_by_dense_index(fastrand::u16(0..air.len() as _));

    let food = Food {
      position: rand_air.position,
      drawable_id: AtomicU16::new(u16::MAX), // Will be initialized in WormApp::init() method
    };

    Self {
      clock,
      air,
      worm,
      food,
      input_worm_direction: None,
    }
  }
}

impl App for WormApp {
  fn get_config(&self) -> AppConfig {
    AppConfig {
      title: "Worm",
      width: consts::APP_SIZE.0 as _,
      height: consts::APP_SIZE.1 as _,
      ..Default::default()
    }
  }

  fn init(&mut self, engine: &mut Engine<'_>) {
    let air_rects = self
      .air
      .get_dense()
      .par_iter()
      .map(|(_, air)| (*air.borrow()).into());

    let top_wall_rects = (0..consts::GRID_SIZE.0)
      .into_par_iter()
      .map(|index| Wall { position: index }.into());

    let bottom_wall_rects = (0..consts::GRID_SIZE.0).into_par_iter().map(|index| {
      Wall {
        position: (consts::GRID_SIZE.1 - 1) * consts::GRID_SIZE.0 + index,
      }
      .into()
    });

    let left_wall_rects = (0..consts::GRID_SIZE.1).into_par_iter().map(|index| {
      Wall {
        position: index * consts::GRID_SIZE.0,
      }
      .into()
    });

    let right_wall_rects = (0..consts::GRID_SIZE.1).into_par_iter().map(|index| {
      Wall {
        position: (index + 1) * consts::GRID_SIZE.0 - 1,
      }
      .into()
    });

    let rects = rayon::iter::once(self.worm.clone().into())
      .chain(rayon::iter::once(self.food.clone().into()))
      .chain(air_rects)
      .chain(top_wall_rects)
      .chain(bottom_wall_rects)
      .chain(left_wall_rects)
      .chain(right_wall_rects)
      .collect();

    engine
      .batch_add_rects(rects)
      .into_par_iter()
      .enumerate()
      .for_each(|(index, id)| {
        if index == 0 {
          self.worm.drawable_id.store(id, Ordering::Relaxed);
        } else if index == 1 {
          self.food.drawable_id.store(id, Ordering::Relaxed);
        } else if index >= 2 && index < 2 + self.air.len() {
          let (_, air) = &self.air.get_dense()[index - 2];
          air.borrow_mut().drawable_id = id;
        }
      });
  }

  fn process_event(&mut self, event: Event) {
    let Event::KeyDown {
      keycode: Some(keycode),
      ..
    } = event
    else {
      return;
    };

    match keycode {
      Keycode::Up | Keycode::W => {
        if !matches!(self.worm.direction, Direction::Down) {
          self.input_worm_direction = Some(Direction::Up);
        }
      }
      Keycode::Right | Keycode::D => {
        if !matches!(self.worm.direction, Direction::Left) {
          self.input_worm_direction = Some(Direction::Right);
        }
      }
      Keycode::Down | Keycode::S => {
        if !matches!(self.worm.direction, Direction::Up) {
          self.input_worm_direction = Some(Direction::Down);
        }
      }
      Keycode::Left | Keycode::A => {
        if !matches!(self.worm.direction, Direction::Right) {
          self.input_worm_direction = Some(Direction::Left);
        }
      }
      _ => {}
    }
  }

  fn update(&mut self, dt: f32, engine: &mut Engine<'_>) {
    if !self.clock.update(dt) {
      return;
    }

    if let Some(input_worm_direction) = self.input_worm_direction.take() {
      self.worm.direction = input_worm_direction;
    }

    let new_worm_position = match self.worm.direction {
      Direction::Up => self.worm.position - consts::GRID_SIZE.0,
      Direction::Right => self.worm.position + 1,
      Direction::Down => self.worm.position + consts::GRID_SIZE.0,
      Direction::Left => self.worm.position - 1,
    };

    let mut air_to_move = self.air.remove(self.worm.position);
    air_to_move.position = self.worm.position;
    self.air.push(air_to_move);
    self.worm.position = new_worm_position;

    engine.update_rect(*self.worm.drawable_id.get_mut(), self.worm.clone().into());
    engine.update_rect(air_to_move.drawable_id, air_to_move.into());
  }
}
