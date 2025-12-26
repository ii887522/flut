use crate::{renderers::RendererRef, utils::AudioManager};

pub struct Context<'ctx> {
  pub audio_manager: &'ctx mut Option<AudioManager>,
  pub renderer: RendererRef<'ctx>,
}
