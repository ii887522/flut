use crate::{AudioManager, renderers::RendererRef};

pub struct Context<'ctx> {
  pub audio_manager: &'ctx mut Option<AudioManager>,
  pub renderer: RendererRef<'ctx>,
}
