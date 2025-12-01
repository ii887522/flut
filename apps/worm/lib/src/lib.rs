#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod consts;
mod level;
mod models;
mod utils;

use crate::{
  level::Level,
  models::{Direction, GameCell, GameCellType},
};
use flut::{Context, Event, Keycode};
use kira::{
  Tween,
  sound::{FromFileError, streaming::StreamingSoundHandle},
};

// Shake settings
const SHAKE_DURATION: f32 = 0.5;
const SHAKE_STRENGTH: f32 = 64.0;

pub struct Game {
  level: Option<Level>,
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
      level: None,
      worm_direction: Direction::rand(),
      input_worm_direction: None,
      worm_move_music: None,
      worm_dead: false,
      accum: 0.0,
      shake_accum: 0.0,
    }
  }

  fn grow_worm(&mut self, context: &mut Context<'_>, new_worm_head_position: u16) {
    if let Some(audio_manager) = context.audio_manager {
      audio_manager.play_sound("assets/worm/audios/eat.mp3");
    }

    let level = self.level.as_mut().unwrap();
    level.grow_worm(context, new_worm_head_position);
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

  game.level = Some(Level::new(&mut context));
  let level = game.level.as_mut().unwrap();
  level.spawn_food(&mut context);
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

  if game.accum < 1.0 / consts::UPDATES_PER_SECOND {
    return;
  }

  game.accum -= 1.0 / consts::UPDATES_PER_SECOND;

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

  let level = game.level.as_mut().unwrap();
  let new_worm_head_position = level.calc_new_worm_head_position(game.worm_direction);

  match level.get_grid_cell(new_worm_head_position) {
    GameCellType::Air => level.move_worm(&mut context, new_worm_head_position),
    GameCellType::Wall | GameCellType::Worm => game.kill_worm(&mut context),
    GameCellType::Food => {
      level.spawn_food(&mut context);
      game.grow_worm(&mut context, new_worm_head_position);
    }
  }
}
