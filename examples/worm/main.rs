#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod consts;
mod models;

use flut::{
  App, AppConfig, Clock, Engine, Transition, app,
  collections::SparseVec,
  gfx::Shake,
  models::{AudioReq, Glass, Rect, RoundRect, Text},
};
use models::{Air, Direction, Food, Score, Wall, Worm};
use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};
use std::{
  collections::VecDeque,
  mem,
  sync::atomic::{AtomicU16, Ordering},
};

fn main() {
  app::run(WormApp::new());
}

enum State {
  Playing,
  DeadShaking(Shake),
  DeadShowingDialog(Glass),
  Pending,
}

struct WormApp {
  clock: Clock,
  air: SparseVec<Air>,
  worm: VecDeque<Worm>, // First element represents head, last element represents tail
  food: Food,
  score: Score,
  state: State,
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
      direction: Direction::rand(),
      drawable_id: AtomicU16::new(u16::MAX), // Will be initialized in WormApp::init() method
    });

    let score = Score {
      score: 0,
      drawable_id: u16::MAX, // Will be initialized in WormApp::init() method
    };

    Self {
      clock,
      air,
      worm,
      food: Food::default(), // Will be initialized in WormApp::init() method
      score,
      state: State::Playing,
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

      engine.update_rect(
        *self.food.drawable_id.get_mut(),
        Rect::from(self.food.clone()),
      );
    } else {
      self.food.position = rand_air.position;
      self.food.drawable_id = AtomicU16::new(engine.add_rect(Rect::from(self.food.clone())));
    }
  }

  fn calc_new_worm_position(&self) -> u16 {
    let worm_head = self.worm.front().unwrap();

    match worm_head.direction {
      Direction::Up => worm_head.position - consts::GRID_SIZE.0,
      Direction::Right => worm_head.position + 1,
      Direction::Down => worm_head.position + consts::GRID_SIZE.0,
      Direction::Left => worm_head.position - 1,
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
    engine.update_rect(air_to_move.drawable_id, Rect::from(air_to_move));
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
      Rect::from(worm_cell_to_move.clone()),
    );

    self.worm.push_front(worm_cell_to_move);
  }

  fn kill_worm(&mut self, engine: &mut Engine<'_>) {
    let _ = engine.get_audio_tx().send(AudioReq::StopMusic);

    let _ = engine.get_audio_tx().send(AudioReq::PlaySound {
      file_path: "assets/audio/dead.mp3",
    });

    self.state = State::DeadShaking(Shake::new(64.0, 0.5, 30.0));
  }

  fn grow_worm(&mut self, engine: &mut Engine<'_>) {
    let _ = engine.get_audio_tx().send(AudioReq::PlaySound {
      file_path: "assets/audio/eat.mp3",
    });

    let new_position = self.calc_new_worm_position();
    let worm_head = self.worm.front().unwrap();

    let mut new_worm_head = Worm {
      position: new_position,
      direction: worm_head.direction,
      drawable_id: AtomicU16::new(u16::MAX),
    };

    new_worm_head.drawable_id = AtomicU16::new(engine.add_rect(Rect::from(new_worm_head.clone())));
    self.worm.push_front(new_worm_head);
  }

  fn add_score(&mut self, engine: &mut Engine<'_>) {
    self.score.score += 1;
    engine.remove_text(self.score.drawable_id);
    self.score.drawable_id = engine.add_text(Text::from(self.score));
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
    let _ = engine.get_audio_tx().send(AudioReq::LoadSound {
      file_path: "assets/audio/dead.mp3",
    });

    let _ = engine.get_audio_tx().send(AudioReq::LoadSound {
      file_path: "assets/audio/eat.mp3",
    });

    let _ = engine.get_audio_tx().send(AudioReq::LoadMusic {
      file_path: "assets/audio/move.mp3",
    });

    let _ = engine.get_audio_tx().send(AudioReq::PlayMusic {
      file_path: "assets/audio/move.mp3",
      volume: 32,
    });

    let air_rects = self
      .air
      .get_dense()
      .par_iter()
      .map(|(_, air)| Rect::from(*air.borrow()));

    let top_wall_rects = (0..consts::GRID_SIZE.0)
      .into_par_iter()
      .map(|index| Rect::from(Wall { position: index }));

    let bottom_wall_rects = (0..consts::GRID_SIZE.0).into_par_iter().map(|index| {
      Rect::from(Wall {
        position: (consts::GRID_SIZE.1 - 1) * consts::GRID_SIZE.0 + index,
      })
    });

    let left_wall_rects = (0..consts::GRID_SIZE.1).into_par_iter().map(|index| {
      Rect::from(Wall {
        position: index * consts::GRID_SIZE.0,
      })
    });

    let right_wall_rects = (0..consts::GRID_SIZE.1).into_par_iter().map(|index| {
      Rect::from(Wall {
        position: (index + 1) * consts::GRID_SIZE.0 - 1,
      })
    });

    let worm_head = self.worm.front().unwrap();
    let worm_head_rect = Rect::from(worm_head.clone());

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
    self.score.drawable_id = engine.add_text(Text::from(self.score));
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

    match keycode {
      Keycode::Up | Keycode::W => {
        if !matches!(worm_head.direction, Direction::Down) {
          self.input_worm_direction = Some(Direction::Up);
        }
      }
      Keycode::Right | Keycode::D => {
        if !matches!(worm_head.direction, Direction::Left) {
          self.input_worm_direction = Some(Direction::Right);
        }
      }
      Keycode::Down | Keycode::S => {
        if !matches!(worm_head.direction, Direction::Up) {
          self.input_worm_direction = Some(Direction::Down);
        }
      }
      Keycode::Left | Keycode::A => {
        if !matches!(worm_head.direction, Direction::Right) {
          self.input_worm_direction = Some(Direction::Left);
        }
      }
      _ => {}
    }
  }

  fn update(&mut self, dt: f32, engine: &mut Engine<'_>) {
    match mem::replace(&mut self.state, State::Pending) {
      State::Playing => {
        self.state = State::Playing;

        if !self.clock.update(dt) {
          return;
        }

        if let Some(input_worm_direction) = self.input_worm_direction.take() {
          let worm_head = self.worm.front_mut().unwrap();
          worm_head.direction = input_worm_direction;
        }

        if self.will_eat_food() {
          self.spawn_food(engine);
          self.grow_worm(engine);
          self.add_score(engine);
          return;
        }

        if self.will_hit_obstacle() {
          self.kill_worm(engine);
          return;
        }

        self.move_air(engine);
        self.move_worm(engine);
      }
      State::DeadShaking(shake) => {
        if let Some(shake) = shake.update(dt, engine) {
          self.state = State::DeadShaking(shake);
          return;
        }

        engine.set_camera_position((0.0, 0.0));

        let mut glass = Glass {
          size: (consts::APP_SIZE.0 as _, consts::APP_SIZE.1 as _),
          alpha: Transition::new(0.0, 128.0, 0.25),
          drawable_id: u16::MAX,
        };

        glass.drawable_id = engine.add_rect(Rect::from(glass));

        engine.add_round_rect(RoundRect::new(
          (
            ((consts::APP_SIZE.0 - consts::DIALOG_SIZE.0) >> 1) as _,
            ((consts::APP_SIZE.1 - consts::DIALOG_SIZE.1) >> 1) as _,
          ),
          (consts::DIALOG_SIZE.0 as _, consts::DIALOG_SIZE.1 as _),
          (255, 0, 0, 255),
          8.0,
        ));

        self.state = State::DeadShowingDialog(glass);
      }
      State::DeadShowingDialog(mut glass) => {
        glass.alpha.update(dt);
        engine.update_rect(glass.drawable_id, Rect::from(glass));
        self.state = State::DeadShowingDialog(glass);
      }
      State::Pending => unreachable!("Unexpected Pending state"),
    };
  }
}
