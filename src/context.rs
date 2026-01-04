use crate::{renderers::RendererRef, utils::AudioManager};

pub struct Context<'ctx> {
  pub audio_manager: &'ctx mut Option<AudioManager>,
  pub renderer: RendererRef<'ctx>,
  pub window_size: (u32, u32),
  pub window_content_scale: f32,
}
