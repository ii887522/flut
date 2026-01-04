use crate::{
  models::Model,
  pipelines::glyph_pipeline::Glyph,
  renderers::{
    ModelRenderer, Renderer,
    renderer::{self, FinishError},
  },
};

#[derive(Clone, Copy)]
pub struct Rect {
  pub position: (f32, f32, u8),
  pub size: (f32, f32),
  pub color: (u8, u8, u8, u8),
}

impl Model for Rect {
  type PipelineModel = Glyph;

  #[inline]
  fn get_renderers_mut(
    renderer: &mut Result<Renderer<renderer::Created>, FinishError>,
  ) -> &mut [ModelRenderer<Self::PipelineModel>] {
    match renderer {
      Ok(renderer) => renderer.get_text_renderer().get_glyph_renderers_mut(),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_text_renderer().get_glyph_renderers_mut()
      }
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
