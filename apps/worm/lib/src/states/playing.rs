use crate::{Direction, Game, GameCellType, State, consts, states::Shaking};
use flut::{
  Context,
  models::{Align, Text},
  utils::Clock,
};
use kira::{
  Tween,
  sound::{FromFileError, streaming::StreamingSoundHandle},
};

pub(crate) struct Playing {
  worm_move_music: Option<StreamingSoundHandle<FromFileError>>,
  clock: Clock,
}

impl Playing {
  pub(super) fn new(context: &mut Context<'_>) -> Self {
    // Worm start moving
    let worm_move_music = if let Some(audio_manager) = context.audio_manager
      && let Some(mut worm_move_music) =
        audio_manager.play_music("assets/worm/audios/worm_move.mp3")
    {
      worm_move_music.set_loop_region(0.2..);
      worm_move_music.set_volume(-10.0, Tween::default());
      Some(worm_move_music)
    } else {
      None
    };

    let clock = Clock::new(1.0 / consts::UPDATES_PER_SECOND);

    Self {
      worm_move_music,
      clock,
    }
  }

  fn move_worm(
    self,
    game: &mut Game,
    context: &mut Context<'_>,
    new_worm_head_position: u16,
  ) -> Self {
    let level = game.level.as_mut().unwrap();
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
    self
  }

  fn grow_worm(
    self,
    game: &mut Game,
    context: &mut Context<'_>,
    new_worm_head_position: u16,
  ) -> Self {
    if let Some(audio_manager) = context.audio_manager {
      audio_manager.play_sound("assets/worm/audios/eat.mp3");
    }

    let level = game.level.as_mut().unwrap();
    let score_render_id = game.score_render_id.unwrap();

    level
      .get_worm_positions_mut()
      .push_front(new_worm_head_position);

    level.set_grid_cell(context, new_worm_head_position, GameCellType::Worm);

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

    self
  }

  fn kill_worm(mut self, game: &Game, context: &mut Context<'_>) -> Shaking {
    if let Some(audio_manager) = context.audio_manager {
      audio_manager.play_sound("assets/worm/audios/hit.wav");
    }

    if let Some(mut worm_move_music) = self.worm_move_music.take() {
      worm_move_music.stop(Tween::default());
    }

    let level = game.level.as_ref().unwrap();
    Shaking::new(level.get_worm_positions().len() - 1)
  }

  pub(crate) fn update(mut self, game: &mut Game, dt: f32, context: &mut Context<'_>) -> State {
    if !self.clock.update(dt) {
      return State::Playing(self);
    }

    if let Some(input_worm_direction) = game.input_worm_direction.take() {
      game.worm_direction = input_worm_direction;
    }

    let (game_cell_type, new_worm_head_position) = {
      let level = game.level.as_ref().unwrap();
      let worm_head_position = *level.get_worm_positions().front().unwrap();

      let new_worm_head_position = match game.worm_direction {
        Direction::Up => worm_head_position - consts::GRID_CELL_COUNTS.0,
        Direction::Right => worm_head_position + 1,
        Direction::Down => worm_head_position + consts::GRID_CELL_COUNTS.0,
        Direction::Left => worm_head_position - 1,
      };

      (
        level.get_grid_cell(new_worm_head_position),
        new_worm_head_position,
      )
    };

    match game_cell_type {
      GameCellType::Air => State::Playing(self.move_worm(game, context, new_worm_head_position)),
      GameCellType::Wall | GameCellType::Worm => State::Shaking(self.kill_worm(game, context)),
      GameCellType::Food => {
        game.spawn_food(context);
        State::Playing(self.grow_worm(game, context, new_worm_head_position))
      }
    }
  }
}
