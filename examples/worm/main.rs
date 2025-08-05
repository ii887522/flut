#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! A Snake-like game implementation where the player controls a worm that grows by eating food
//! while avoiding walls and itself.

mod consts;
mod models;

use crate::models::{Air, Direction, Food, Score, Wall, Worm};
use flut::{
  App, Clock, Context,
  animations::Shake,
  app,
  collections::SparseSet,
  models::{AudioReq, Rect, Text},
};
use mimalloc::MiMalloc;
use rayon::{iter::Either, prelude::*};
use sdl2::{event::Event, keyboard::Keycode};
use std::collections::VecDeque;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
  app::run(WormGame::new());
}

struct WormGame {
  /// Empty cells that can be moved into or contain food
  airs: SparseSet<Air>,
  /// The game boundaries and obstacles
  walls: Box<[Wall]>,
  /// The worm body segments, stored front (tail) to back (head) for efficient movement
  worms: VecDeque<Worm>,
  /// The current food item on the game board
  food: Option<Food>,
  /// The player's current score
  score: Score,
  /// Whether the worm has collided with a wall or itself
  worm_dead: bool,
  /// Controls the game update rate
  clock: Clock,
  /// Current screen shake animation, if any
  shake: Option<Shake>,
  /// The next direction the worm should turn, if any
  next_worm_direction: Option<Direction>,
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
            direction: Direction::rand(),
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

    let score = Score {
      score: 0,
      drawable_id: u32::MAX,
    };

    Self {
      airs: SparseSet::from_par_iter(airs),
      walls: walls.into_boxed_slice(),
      worms: VecDeque::from_iter(worms),
      food: None,
      score,
      worm_dead: false,
      clock: Clock::new(1.0 / consts::UPDATES_PER_SECOND),
      shake: None,
      next_worm_direction: None,
    }
  }

  fn move_worm(&mut self, context: &mut Context<'_>) -> bool {
    let worm_head = *self.worms.back().unwrap();
    let worm_head_next_position = worm_head.calc_next_position();

    let Some(remove_air_resp) = self.airs.remove(worm_head_next_position) else {
      return false;
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

    context
      .renderer
      .update_rect(worm_tail.drawable_id, Rect::from(new_worm_head));

    context
      .renderer
      .update_rect(air.drawable_id, Rect::from(new_air));

    true
  }

  fn will_eat_food(&self) -> bool {
    let worm_head = self.worms.back().unwrap();
    let food = self.food.unwrap();
    worm_head.calc_next_position() == food.position
  }

  fn grow_worm(&mut self, context: &mut Context<'_>) {
    context
      .audio_tx
      .send(AudioReq::PlaySound("assets/worm/audios/eat.mp3".into()))
      .unwrap();

    let worm_head = self.worms.back().unwrap();
    let food = self.food.unwrap();

    let new_worm_head = Worm {
      position: food.position,
      drawable_id: food.drawable_id,
      direction: worm_head.direction,
    };

    self.worms.push_back(new_worm_head);

    context
      .renderer
      .update_rect(food.drawable_id, Rect::from(new_worm_head));
  }

  fn spawn_food(&mut self, context: &mut Context<'_>) {
    let Some(&rand_air_position) = fastrand::choice(self.airs.get_dense_ids()) else {
      return;
    };

    let remove_air_resp = self.airs.remove(rand_air_position).unwrap();

    let food = Food {
      position: rand_air_position,
      drawable_id: remove_air_resp.item.drawable_id,
    };

    self.food = Some(food);

    context
      .renderer
      .update_rect(remove_air_resp.item.drawable_id, Rect::from(food));
  }

  fn add_score(&mut self, context: &mut Context<'_>) {
    self.score.score += 1;

    context
      .renderer
      .remove_text(self.score.drawable_id)
      .unwrap();

    self.score.drawable_id = context.renderer.add_text(Text::from(self.score));
  }

  fn kill_worm(&mut self, context: &Context<'_>) {
    context
      .audio_tx
      .send(AudioReq::PlaySound("assets/worm/audios/dead.mp3".into()))
      .unwrap();

    context.audio_tx.send(AudioReq::HaltMusic).unwrap();
    self.worm_dead = true;
    self.shake = Some(Shake::new(1.0 / consts::SHAKE_PER_SECOND, 0.5, 50.0));
  }
}

impl App for WormGame {
  fn get_config(&self) -> app::Config {
    app::Config {
      title: "Worm".into(),
      size: consts::APP_SIZE,
      favicon_path: "assets/worm/favicon.png".into(),
      font_path: "assets/worm/fonts/arial.ttf".into(),
      ..Default::default()
    }
  }

  fn init(&mut self, mut context: Context<'_>) {
    context
      .audio_tx
      .send(AudioReq::PlayMusic("assets/worm/audios/moving.mp3".into()))
      .unwrap();

    let rect_ids = context.renderer.add_rects(
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

    self.score.drawable_id = context.renderer.add_text(Text::from(self.score));
    self.spawn_food(&mut context);
  }

  fn process_event(&mut self, event: Event) {
    let worm_head = self.worms.back().unwrap();

    match event {
      Event::KeyDown {
        keycode: Some(Keycode::W | Keycode::Up),
        ..
      } if !matches!(worm_head.direction, Direction::Down) => {
        self.next_worm_direction = Some(Direction::Up);
      }
      Event::KeyDown {
        keycode: Some(Keycode::D | Keycode::Right),
        ..
      } if !matches!(worm_head.direction, Direction::Left) => {
        self.next_worm_direction = Some(Direction::Right);
      }
      Event::KeyDown {
        keycode: Some(Keycode::S | Keycode::Down),
        ..
      } if !matches!(worm_head.direction, Direction::Up) => {
        self.next_worm_direction = Some(Direction::Down);
      }
      Event::KeyDown {
        keycode: Some(Keycode::A | Keycode::Left),
        ..
      } if !matches!(worm_head.direction, Direction::Right) => {
        self.next_worm_direction = Some(Direction::Left);
      }
      _ => {}
    }
  }

  fn update(&mut self, dt: f32, mut context: Context<'_>) {
    if let Some(shake) = self.shake.take() {
      self.shake = shake.update(dt, &mut context);
    }

    if !self.clock.update(dt) || self.worm_dead {
      return;
    }

    if let Some(next_worm_direction) = self.next_worm_direction.take() {
      let worm_head = self.worms.back_mut().unwrap();
      worm_head.direction = next_worm_direction;
    }

    if self.move_worm(&mut context) {
      return;
    }

    if self.will_eat_food() {
      self.grow_worm(&mut context);
      self.spawn_food(&mut context);
      self.add_score(&mut context);
    } else {
      // Hit wall or itself
      self.kill_worm(&context);
    }
  }
}
