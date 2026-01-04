#![deny(clippy::all, elided_lifetimes_in_paths)]
#![allow(clippy::needless_lifetimes, clippy::too_many_arguments)]

pub mod consts;
mod level;
mod models;
mod states;
mod utils;

use crate::{
  level::Level,
  models::{Countdown, Direction, GameCell, GameCellType},
  states::{Preparing, State},
};
use flut::{
  Context,
  event::Event,
  keyboard::Keycode,
  models::{Align, Text},
  renderers::renderer_ref,
};
use std::mem;

pub struct Game {
  level: Option<Level>,
  score_render_id: Option<renderer_ref::Id>,
  worm_direction: Direction,
  input_worm_direction: Option<Direction>,
  state: State,
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
      worm_direction: Direction::rand(),
      input_worm_direction: None,
      state: State::Preparing(Preparing::new()),
    }
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

  let State::Preparing(preparing) = mem::replace(&mut game.state, State::Pending) else {
    unreachable!("game.state is not Preparing");
  };

  game.state = State::Preparing(preparing.init(&mut context));
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn process_event(game: &mut Game, event: Event, context: Context<'_>) {
  match event {
    Event::KeyDown {
      keycode: Some(Keycode::W | Keycode::Up),
      ..
    } if game.worm_direction != Direction::Down || matches!(game.state, State::Preparing(_)) => {
      game.input_worm_direction = Some(Direction::Up)
    }
    Event::KeyDown {
      keycode: Some(Keycode::D | Keycode::Right),
      ..
    } if game.worm_direction != Direction::Left || matches!(game.state, State::Preparing(_)) => {
      game.input_worm_direction = Some(Direction::Right)
    }
    Event::KeyDown {
      keycode: Some(Keycode::S | Keycode::Down),
      ..
    } if game.worm_direction != Direction::Up || matches!(game.state, State::Preparing(_)) => {
      game.input_worm_direction = Some(Direction::Down)
    }
    Event::KeyDown {
      keycode: Some(Keycode::A | Keycode::Left),
      ..
    } if game.worm_direction != Direction::Right || matches!(game.state, State::Preparing(_)) => {
      game.input_worm_direction = Some(Direction::Left)
    }
    _ => {}
  }

  game.state = match mem::replace(&mut game.state, State::Pending) {
    State::DialogShown(dialog_shown) => {
      State::DialogShown(dialog_shown.process_event(event, &context))
    }
    State::Pending => unreachable!("game.state is State::Pending"),
    state => state,
  };
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub extern "Rust" fn update(game: &mut Game, dt: f32, mut context: Context<'_>) {
  game.state = match mem::replace(&mut game.state, State::Pending) {
    State::Preparing(preparing) => preparing.update(game, dt, &mut context),
    State::Playing(playing) => playing.update(game, dt, &mut context),
    State::Shaking(shaking) => shaking.update(dt, &mut context),
    State::ShowingDialog(showing_dialog) => showing_dialog.update(dt, &mut context),
    State::DialogShown(dialog_shown) => State::DialogShown(dialog_shown.update(dt, &mut context)),
    State::Pending => unreachable!("game.state is State::Pending"),
  };
}
