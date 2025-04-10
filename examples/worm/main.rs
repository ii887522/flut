#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod consts;
mod models;

use flut::{App, AppConfig, Clock, Engine, app, collections::SparseVec};
use models::{Air, Direction, Food, Wall, Worm};
use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};
use std::{
  collections::VecDeque,
  sync::atomic::{AtomicU16, Ordering},
};

fn main() {
  app::run(WormApp::new());
}

struct WormApp {
  clock: Clock,
  air: SparseVec<Air>,
  worm: VecDeque<Worm>, // First element represents head, last element represents tail
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

    let mut worm =
      VecDeque::with_capacity(((consts::GRID_SIZE.0 - 2) * (consts::GRID_SIZE.1 - 2)) as _);

    worm.push_front(Worm {
      position: consts::INITIAL_WORM_POSITION_1D,
      direction: Some(Direction::rand()),
      drawable_id: AtomicU16::new(u16::MAX), // Will be initialized in WormApp::init() method
    });

    Self {
      clock,
      air,
      worm,
      food: Food::default(), // Will be initialized in WormApp::init() method
      input_worm_direction: None,
    }
  }

  fn spawn_food(&mut self, engine: &mut Engine<'_>) {
    let rand_air = self
      .air
      .remove_by_dense_index(fastrand::u16(0..self.air.len() as _));

    engine.remove_rect(rand_air.drawable_id);

    // Has spawn food before
    if self.food.position < u16::MAX {
      self.food.position = rand_air.position;
      engine.update_rect(*self.food.drawable_id.get_mut(), self.food.clone().into());
    } else {
      self.food.position = rand_air.position;
      self.food.drawable_id = AtomicU16::new(engine.add_rect(self.food.clone().into()));
    }
  }

  fn calc_new_worm_position(&self) -> u16 {
    let worm_head = self.worm.front().unwrap();

    match worm_head.direction {
      Some(Direction::Up) => worm_head.position - consts::GRID_SIZE.0,
      Some(Direction::Right) => worm_head.position + 1,
      Some(Direction::Down) => worm_head.position + consts::GRID_SIZE.0,
      Some(Direction::Left) => worm_head.position - 1,
      None => worm_head.position,
    }
  }

  fn will_eat_food(&self) -> bool {
    self.calc_new_worm_position() == self.food.position
  }

  fn will_hit_obstacle(&self) -> bool {
    !self.air.contains(self.calc_new_worm_position())
  }

  fn move_air(&mut self, engine: &mut Engine<'_>) {
    let worm_tail = self.worm.back().unwrap();
    let mut air_to_move = self.air.remove(self.calc_new_worm_position());
    air_to_move.position = worm_tail.position;
    self.air.push_by_id(worm_tail.position, air_to_move);
    engine.update_rect(air_to_move.drawable_id, air_to_move.into());
  }

  fn move_worm(&mut self, engine: &mut Engine<'_>) {
    let new_position = self.calc_new_worm_position();
    let mut worm_cell_to_move = self.worm.pop_back().unwrap();

    let worm_head = if let Some(worm_head) = self.worm.front() {
      worm_head
    } else {
      &worm_cell_to_move
    };

    worm_cell_to_move.direction = worm_head.direction;
    worm_cell_to_move.position = new_position;

    engine.update_rect(
      *worm_cell_to_move.drawable_id.get_mut(),
      worm_cell_to_move.clone().into(),
    );

    self.worm.push_front(worm_cell_to_move);
  }

  fn kill_worm(&mut self) {
    let worm_head = self.worm.front_mut().unwrap();
    worm_head.direction = None;
  }

  fn grow_worm(&mut self, engine: &mut Engine<'_>) {
    let new_position = self.calc_new_worm_position();
    let worm_head = self.worm.front().unwrap();

    let mut new_worm_head = Worm {
      position: new_position,
      direction: worm_head.direction,
      drawable_id: AtomicU16::new(u16::MAX),
    };

    new_worm_head.drawable_id = AtomicU16::new(engine.add_rect(new_worm_head.clone().into()));
    self.worm.push_front(new_worm_head);
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

    let worm_head = self.worm.front().unwrap();
    let worm_head_rect = worm_head.clone().into();

    let rects = rayon::iter::once(worm_head_rect)
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
          worm_head.drawable_id.store(id, Ordering::Relaxed);
        } else if index >= 1 && index < 1 + self.air.len() {
          let (_, air) = &self.air.get_dense()[index - 1];
          air.borrow_mut().drawable_id = id;
        }
      });

    self.spawn_food(engine);
  }

  fn process_event(&mut self, event: Event) {
    let Event::KeyDown {
      keycode: Some(keycode),
      ..
    } = event
    else {
      return;
    };

    let worm_head = self.worm.front().unwrap();

    let Some(worm_head_direction) = worm_head.direction else {
      return;
    };

    match keycode {
      Keycode::Up | Keycode::W => {
        if !matches!(worm_head_direction, Direction::Down) {
          self.input_worm_direction = Some(Direction::Up);
        }
      }
      Keycode::Right | Keycode::D => {
        if !matches!(worm_head_direction, Direction::Left) {
          self.input_worm_direction = Some(Direction::Right);
        }
      }
      Keycode::Down | Keycode::S => {
        if !matches!(worm_head_direction, Direction::Up) {
          self.input_worm_direction = Some(Direction::Down);
        }
      }
      Keycode::Left | Keycode::A => {
        if !matches!(worm_head_direction, Direction::Right) {
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

    let worm_head = self.worm.front_mut().unwrap();

    if worm_head.direction.is_none() {
      return;
    }

    if let Some(input_worm_direction) = self.input_worm_direction.take() {
      worm_head.direction = Some(input_worm_direction);
    }

    if self.will_eat_food() {
      self.spawn_food(engine);
      self.grow_worm(engine);
      return;
    }

    if self.will_hit_obstacle() {
      self.kill_worm();
      return;
    }

    self.move_air(engine);
    self.move_worm(engine);
  }
}
