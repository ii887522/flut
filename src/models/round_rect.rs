use crate::{
  models::Model,
  pipelines::round_rect_pipeline,
  renderers::{
    ModelRenderer, Renderer,
    renderer::{self, FinishError},
  },
};

#[derive(Clone, Copy)]
pub struct RoundRect {
  pub position: (f32, f32, u8),
  pub size: (f32, f32),
  pub radius: f32,
  pub color: (u8, u8, u8, u8),
}

impl Model for RoundRect {
  type PipelineModel = round_rect_pipeline::RoundRect;

  #[inline]
  fn get_renderers_mut(
    renderer: &mut Result<Renderer<renderer::Created>, FinishError>,
  ) -> &mut [ModelRenderer<Self::PipelineModel>] {
    match renderer {
      Ok(renderer) => renderer.get_round_rect_renderers(),
      Err(FinishError::WindowMinimized(renderer)) => renderer.get_round_rect_renderers(),
    }
  }

  #[inline]
  fn get_z(&self) -> u8 {
    self.position.2
  }

  #[inline]
  fn into_pipeline_model(self) -> Self::PipelineModel {
    self.into()
  }
}
