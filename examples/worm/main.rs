#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod consts;
mod models;

use flut::{
  App, AppConfig, Clock, Engine, app,
  collections::SparseVec,
  gfx::Shake,
  models::{AudioReq, IconName, Rect, Text},
  widgets::{Dialog, dialog::DialogButton},
};
use models::{Air, Direction, Food, Score, Wall, Worm};
use rayon::prelude::*;
use sdl2::{event::Event, keyboard::Keycode};
use std::{
  cell::RefCell,
  collections::VecDeque,
  mem,
  rc::Rc,
  sync::atomic::{AtomicU16, Ordering},
};

fn main() {
  app::run(WormApp::new());
}

struct WormApp {
  clock: Clock,
  game: Rc<RefCell<WormGame>>,
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
      game: Rc::new(RefCell::new(WormGame {
        air,
        worm,
        food: Food::default(), // Will be initialized in WormApp::init() method
        score,
        state: State::Playing,
      })),
      input_worm_direction: None,
    }
  }

  fn spawn_food(&mut self, engine: &mut Engine) {
    let mut game = self.game.borrow_mut();
    let air_count = game.air.len() as _;
    let rand_air = game.air.remove_by_dense_index(fastrand::u16(0..air_count));

    engine.remove_rect(rand_air.drawable_id);

    // Has spawn food before
    if game.food.position < u16::MAX {
      game.food.position = rand_air.position;

      engine.update_rect(
        *game.food.drawable_id.get_mut(),
        Rect::from(game.food.clone()),
      );
    } else {
      game.food.position = rand_air.position;
      game.food.drawable_id = AtomicU16::new(engine.add_rect(Rect::from(game.food.clone())));
    }
  }

  fn calc_new_worm_position(&self) -> u16 {
    let game = self.game.borrow();
    let worm_head = game.worm.front().unwrap();

    match worm_head.direction {
      Direction::Up => worm_head.position - consts::GRID_SIZE.0,
      Direction::Right => worm_head.position + 1,
      Direction::Down => worm_head.position + consts::GRID_SIZE.0,
      Direction::Left => worm_head.position - 1,
    }
  }

  fn will_eat_food(&self) -> bool {
    self.calc_new_worm_position() == self.game.borrow().food.position
  }

  fn will_hit_obstacle(&self) -> bool {
    !self
      .game
      .borrow()
      .air
      .contains(self.calc_new_worm_position())
  }

  fn move_air(&mut self, engine: &mut Engine) {
    let new_worm_position = self.calc_new_worm_position();
    let mut game = self.game.borrow_mut();
    let mut air_to_move = game.air.remove(new_worm_position);
    let worm_tail = game.worm.back().unwrap().clone();
    air_to_move.position = worm_tail.position;
    game.air.push_by_id(worm_tail.position, air_to_move);
    engine.update_rect(air_to_move.drawable_id, Rect::from(air_to_move));
  }

  fn move_worm(&mut self, engine: &mut Engine) {
    let new_position = self.calc_new_worm_position();
    let mut game = self.game.borrow_mut();
    let mut worm_cell_to_move = game.worm.pop_back().unwrap();

    let worm_head = if let Some(worm_head) = game.worm.front() {
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

    game.worm.push_front(worm_cell_to_move);
  }

  fn kill_worm(&mut self, engine: &mut Engine) {
    let _ = engine.get_audio_tx().send(AudioReq::StopMusic);

    let _ = engine.get_audio_tx().send(AudioReq::PlaySound {
      file_path: "assets/audio/dead.mp3",
    });

    self.game.borrow_mut().state = State::DeadShaking(Shake::new(64.0, 0.5, 30.0));
  }

  fn grow_worm(&mut self, engine: &mut Engine) {
    let _ = engine.get_audio_tx().send(AudioReq::PlaySound {
      file_path: "assets/audio/eat.mp3",
    });

    let new_position = self.calc_new_worm_position();
    let mut game = self.game.borrow_mut();
    let worm_head = game.worm.front().unwrap();

    let mut new_worm_head = Worm {
      position: new_position,
      direction: worm_head.direction,
      drawable_id: AtomicU16::new(u16::MAX),
    };

    new_worm_head.drawable_id = AtomicU16::new(engine.add_rect(Rect::from(new_worm_head.clone())));
    game.worm.push_front(new_worm_head);
  }

  fn add_score(&mut self, engine: &mut Engine) {
    let mut game = self.game.borrow_mut();
    game.score.score += 1;
    engine.remove_text(game.score.drawable_id);
    game.score.drawable_id = engine.add_text(Text::from(game.score));
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

  fn init(&mut self, engine: Rc<RefCell<Engine>>) {
    let mut engine = engine.borrow_mut();

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

    let mut game = self.game.borrow_mut();

    let air_rects = game
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

    let worm_head = game.worm.front().unwrap();
    let worm_head_rect = Rect::from(worm_head.clone());

    let rects = rayon::iter::once(worm_head_rect)
      .chain(air_rects)
      .chain(top_wall_rects)
      .chain(bottom_wall_rects)
      .chain(left_wall_rects)
      .chain(right_wall_rects)
      .collect();

    let air_count = game.air.len();
    let air = game.air.get_dense();

    engine
      .batch_add_rects(rects)
      .into_par_iter()
      .enumerate()
      .for_each(|(index, id)| {
        if index == 0 {
          worm_head.drawable_id.store(id, Ordering::Relaxed);
        } else if index >= 1 && index < 1 + air_count {
          let (_, air) = &air[index - 1];
          air.borrow_mut().drawable_id = id;
        }
      });

    game.score.drawable_id = engine.add_text(Text::from(game.score));
    drop(game);
    self.spawn_food(&mut engine);
  }

  fn process_event(&mut self, event: Event) {
    let dialog = if let State::DeadShowingDialog(dialog) = &mut self.game.borrow_mut().state {
      Some(dialog.take().unwrap())
    } else {
      None
    };

    if let Some(mut dialog) = dialog {
      dialog.process_event(&event);
      let mut game = self.game.borrow_mut();

      if let State::DeadShowingDialog(None) = game.state {
        game.state = State::DeadShowingDialog(Some(dialog));
      }
    }

    let Event::KeyDown {
      keycode: Some(keycode),
      ..
    } = event
    else {
      return;
    };

    let game = self.game.borrow();
    let worm_head = game.worm.front().unwrap();

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

  fn update(&mut self, dt: f32, engine: Rc<RefCell<Engine>>) {
    let cancel_button_engine = engine.clone();
    let ok_button_engine = engine.clone();
    let mut engine = engine.borrow_mut();
    let mut game = self.game.borrow_mut();

    match mem::replace(&mut game.state, State::Pending) {
      State::Playing => {
        game.state = State::Playing;

        if !self.clock.update(dt) {
          return;
        }

        if let Some(input_worm_direction) = self.input_worm_direction.take() {
          let worm_head = game.worm.front_mut().unwrap();
          worm_head.direction = input_worm_direction;
        }

        drop(game);

        if self.will_eat_food() {
          self.spawn_food(&mut engine);
          self.grow_worm(&mut engine);
          self.add_score(&mut engine);
          return;
        }

        if self.will_hit_obstacle() {
          self.kill_worm(&mut engine);
          return;
        }

        self.move_air(&mut engine);
        self.move_worm(&mut engine);
      }
      State::DeadShaking(shake) => {
        if let Some(shake) = shake.update(dt, &mut engine) {
          game.state = State::DeadShaking(shake);
          return;
        }

        engine.set_camera_position((0.0, 0.0));
        let score = game.score.score;
        let game_rc = self.game.clone();

        let mut dialog = Dialog::new(
          IconName::Skull,
          DialogButton::new(IconName::House)
            .bg_color((0, 0, 128, 255))
            .color((255, 255, 255, 255))
            .label("Home")
            .on_click(Box::new(move || {
              cancel_button_engine.borrow().send_event(Event::Quit {
                timestamp: flut::get_current_timestamp() as _,
              });
            }) as Box<_>)
            .call(),
          DialogButton::new(IconName::RestartAlt)
            .bg_color((128, 0, 0, 255))
            .color((255, 255, 255, 255))
            .label("Play again")
            .on_click(Box::new(move || {
              ok_button_engine.borrow_mut().clear();
              let mut app = Self::new();
              app.init(ok_button_engine.clone());
              let new_game = Rc::into_inner(app.game).unwrap().into_inner();
              game_rc.borrow_mut().copy_from(new_game);
            }) as Box<_>)
            .call(),
        )
        .bg_color((255, 0, 0, 255))
        .color((255, 255, 255, 255))
        .title("Game Over")
        .desc(format!(
          "You scored {score} points. Do you want to try again?"
        ))
        .call();

        dialog.init(&mut engine);
        game.state = State::DeadShowingDialog(Some(Box::new(dialog)));
      }
      State::DeadShowingDialog(Some(mut dialog)) => {
        dialog.update(dt, &mut engine);
        game.state = State::DeadShowingDialog(Some(dialog));
      }
      state => unreachable!("Unexpected: {state:?}"),
    };
  }
}

#[derive(Debug)]
enum State {
  Playing,
  DeadShaking(Shake),
  DeadShowingDialog(Option<Box<Dialog>>),
  Pending,
}

struct WormGame {
  air: SparseVec<Air>,
  worm: VecDeque<Worm>, // First element represents head, last element represents tail
  food: Food,
  score: Score,
  state: State,
}

impl WormGame {
  fn copy_from(&mut self, other: Self) {
    self.air = other.air;
    self.worm = other.worm;
    self.food = other.food;
    self.score = other.score;
    self.state = other.state;
  }
}
