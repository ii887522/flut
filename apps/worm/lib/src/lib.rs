#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod consts;
mod level;
mod models;
mod utils;

use crate::{
  level::Level,
  models::{Countdown, Direction, GameCell, GameCellType},
};
use flut::{
  Context, Event, Keycode,
  models::{Align, Text},
  renderers::renderer_ref,
};
use kira::{
  Tween,
  sound::{FromFileError, streaming::StreamingSoundHandle},
};

// Shake settings
const SHAKE_DURATION: f32 = 0.5;
const SHAKE_STRENGTH: f32 = 64.0;

pub struct Game {
  level: Option<Level>,
  score_render_id: Option<renderer_ref::Id>,
  countdown: Option<Countdown>,
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
      score_render_id: None,
      countdown: None,
      worm_direction: Direction::rand(),
      input_worm_direction: None,
      worm_move_music: None,
      worm_dead: false,
      accum: 0.0,
      shake_accum: 0.0,
    }
  }

  fn update_countdown(&mut self, dt: f32, context: &mut Context<'_>) -> bool {
    let Some(countdown) = self.countdown.as_mut() else {
      return false;
    };

    countdown.accum += dt;

    if countdown.accum < consts::COUNTDOWN_INTERVAL {
      let t = countdown.accum / consts::COUNTDOWN_INTERVAL;

      context.renderer.update_text(
        countdown.render_id,
        Text {
          position: consts::COUNTDOWN_POSITION,
          color: (
            consts::COUNTDOWN_COLOR.0,
            consts::COUNTDOWN_COLOR.1,
            consts::COUNTDOWN_COLOR.1,
            consts::COUNTDOWN_MAX_ALPHA * (1.0 - t),
          ),
          font_size: consts::COUNTDOWN_MIN_FONT_SIZE
            + (t * (consts::COUNTDOWN_MAX_FONT_SIZE - consts::COUNTDOWN_MIN_FONT_SIZE) as f32)
              as u16,
          align: Align::Center,
          text: countdown.countdown.to_string().into(),
        },
      );

      return true;
    }

    countdown.accum -= consts::COUNTDOWN_INTERVAL;
    countdown.countdown -= 1;

    if countdown.countdown > 0 {
      context.renderer.update_text(
        countdown.render_id,
        Text {
          position: consts::COUNTDOWN_POSITION,
          color: (1.0, 1.0, 1.0, 0.5),
          font_size: consts::COUNTDOWN_MIN_FONT_SIZE,
          align: Align::Center,
          text: countdown.countdown.to_string().into(),
        },
      );
    } else {
      // Worm start moving
      if let Some(audio_manager) = context.audio_manager
        && let Some(mut worm_move_music) =
          audio_manager.play_music("assets/worm/audios/worm_move.mp3")
      {
        worm_move_music.set_loop_region(0.2..);
        worm_move_music.set_volume(-10.0, Tween::default());
        self.worm_move_music = Some(worm_move_music);
      }

      context.renderer.remove_text(countdown.render_id);
      self.countdown = None;
    }

    true
  }

  fn move_worm(&mut self, context: &mut Context<'_>, new_worm_head_position: u16) {
    let level = self.level.as_mut().unwrap();
    let worm_tail_position = level.get_worm_positions_mut().pop_back().unwrap();

    level
      .get_worm_positions_mut()
      .push_front(new_worm_head_position);

    level
      .get_air_positions_mut()
      .swap_remove(&new_worm_head_position);

    level.get_air_positions_mut().insert(worm_tail_position);
    level.set_grid_cell(context, worm_tail_position, GameCellType::Air);
    level.set_grid_cell(context, new_worm_head_position, GameCellType::Worm);
  }

  fn grow_worm(&mut self, context: &mut Context<'_>, new_worm_head_position: u16) {
    if let Some(audio_manager) = context.audio_manager {
      audio_manager.play_sound("assets/worm/audios/eat.mp3");
    }

    let level = self.level.as_mut().unwrap();

    level
      .get_worm_positions_mut()
      .push_front(new_worm_head_position);

    level.set_grid_cell(context, new_worm_head_position, GameCellType::Worm);

    let score_render_id = self.score_render_id.unwrap();

    context.renderer.update_text(
      score_render_id,
      Text {
        position: consts::SCORE_POSITION,
        color: consts::SCORE_COLOR,
        font_size: consts::SCORE_FONT_SIZE,
        align: Align::Center,
        text: (level.get_worm_positions().len() - 1).to_string().into(),
      },
    );
  }

  fn spawn_food(&mut self, context: &mut Context<'_>) {
    let level = self.level.as_mut().unwrap();
    let rand_air_position_index = fastrand::usize(..level.get_air_positions().len());

    let air_position_to_remove = level
      .get_air_positions_mut()
      .swap_remove_index(rand_air_position_index)
      .unwrap();

    level.set_grid_cell(context, air_position_to_remove, GameCellType::Food);
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

  fn shake(&mut self, context: &mut Context<'_>) {
    if self.shake_accum < SHAKE_DURATION {
      context.renderer.set_cam_position(Some((
        fastrand::f32() * SHAKE_STRENGTH,
        fastrand::f32() * SHAKE_STRENGTH,
      )));
    } else {
      self.shake_accum = SHAKE_DURATION;
      context.renderer.set_cam_position(None);
    }
  }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn init(game: &mut Game, mut context: Context<'_>) {
  game.level = Some(Level::new(&mut context));
  game.spawn_food(&mut context);

  let score_render_id = context.renderer.add_text(Text {
    position: consts::SCORE_POSITION,
    color: consts::SCORE_COLOR,
    font_size: consts::SCORE_FONT_SIZE,
    align: Align::Center,
    text: "0".into(),
  });

  game.score_render_id = Some(score_render_id);

  let countdown_render_id = context.renderer.add_text(Text {
    position: consts::COUNTDOWN_POSITION,
    color: (1.0, 1.0, 1.0, 0.5),
    font_size: consts::COUNTDOWN_MIN_FONT_SIZE,
    align: Align::Center,
    text: consts::MAX_COUNTDOWN.to_string().into(),
  });

  game.countdown = Some(Countdown {
    countdown: consts::MAX_COUNTDOWN,
    render_id: countdown_render_id,
    accum: 0.0,
  });
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn process_event(game: &mut Game, event: Event) {
  match event {
    Event::KeyDown {
      keycode: Some(Keycode::W | Keycode::Up),
      ..
    } if game.worm_direction != Direction::Down || game.countdown.is_some() => {
      game.input_worm_direction = Some(Direction::Up)
    }
    Event::KeyDown {
      keycode: Some(Keycode::D | Keycode::Right),
      ..
    } if game.worm_direction != Direction::Left || game.countdown.is_some() => {
      game.input_worm_direction = Some(Direction::Right)
    }
    Event::KeyDown {
      keycode: Some(Keycode::S | Keycode::Down),
      ..
    } if game.worm_direction != Direction::Up || game.countdown.is_some() => {
      game.input_worm_direction = Some(Direction::Down)
    }
    Event::KeyDown {
      keycode: Some(Keycode::A | Keycode::Left),
      ..
    } if game.worm_direction != Direction::Right || game.countdown.is_some() => {
      game.input_worm_direction = Some(Direction::Left)
    }
    _ => {}
  }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn update(game: &mut Game, dt: f32, mut context: Context<'_>) {
  if game.update_countdown(dt, &mut context) {
    return;
  }

  game.accum += dt;

  if game.worm_dead {
    game.shake_accum += dt;
  }

  if game.accum < 1.0 / consts::UPDATES_PER_SECOND {
    return;
  }

  game.accum -= 1.0 / consts::UPDATES_PER_SECOND;

  if game.worm_dead {
    game.shake(&mut context);
    return;
  }

  if let Some(input_worm_direction) = game.input_worm_direction.take() {
    game.worm_direction = input_worm_direction;
  }

  let level = game.level.as_mut().unwrap();
  let worm_head_position = *level.get_worm_positions().front().unwrap();

  let new_worm_head_position = match game.worm_direction {
    Direction::Up => worm_head_position - consts::GRID_CELL_COUNTS.0,
    Direction::Right => worm_head_position + 1,
    Direction::Down => worm_head_position + consts::GRID_CELL_COUNTS.0,
    Direction::Left => worm_head_position - 1,
  };

  match level.get_grid_cell(new_worm_head_position) {
    GameCellType::Air => game.move_worm(&mut context, new_worm_head_position),
    GameCellType::Wall | GameCellType::Worm => game.kill_worm(&mut context),
    GameCellType::Food => {
      game.spawn_food(&mut context);
      game.grow_worm(&mut context, new_worm_head_position);
    }
  }
}
