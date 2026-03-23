use crate::{
  glyph_renderer::{GlyphRenderer, IconId, TextId},
  model_sync::ModelSync,
  models::{Model, icon::Icon, text::Text},
  renderer::{Created, Creating, Renderer},
};
use winit::{dpi::LogicalSize, window::Window};

pub struct RendererRef<'render>(&'render mut Result<Renderer<Created>, Renderer<Creating>>);

impl<'render> RendererRef<'render> {
  #[inline]
  pub(super) const fn new(
    renderer: &'render mut Result<Renderer<Created>, Renderer<Creating>>,
  ) -> Self {
    Self(renderer)
  }

  #[must_use]
  #[inline]
  pub const fn get_window(&self) -> &Window {
    match *self.0 {
      Ok(ref renderer) => renderer.get_window(),
      Err(ref renderer) => renderer.get_window(),
    }
  }

  #[must_use]
  #[inline]
  pub fn get_size(&self) -> (f32, f32) {
    let window = self.get_window();
    let LogicalSize { width, height } = window.inner_size().to_logical(window.scale_factor());
    (width, height)
  }

  #[inline]
  fn get_model_sync<M: Model>(&mut self, clipped: bool) -> &mut ModelSync<M> {
    if clipped {
      match *self.0 {
        Ok(ref mut renderer) => M::get_clipped_sync(renderer),
        Err(ref mut renderer) => M::get_clipped_sync(renderer),
      }
    } else {
      match *self.0 {
        Ok(ref mut renderer) => M::get_sync(renderer),
        Err(ref mut renderer) => M::get_sync(renderer),
      }
    }
  }

  #[inline]
  const fn get_glyph_renderer(&self) -> &GlyphRenderer {
    match *self.0 {
      Ok(ref renderer) => renderer.get_glyph_renderer(),
      Err(ref renderer) => renderer.get_glyph_renderer(),
    }
  }

  #[inline]
  const fn get_glyph_renderer_mut(&mut self) -> &mut GlyphRenderer {
    match *self.0 {
      Ok(ref mut renderer) => renderer.get_glyph_renderer_mut(),
      Err(ref mut renderer) => renderer.get_glyph_renderer_mut(),
    }
  }

  #[inline]
  pub fn add_model<M: Model>(&mut self, model: M, clipped: bool) -> u32 {
    self.get_model_sync(clipped).add_model(model)
  }

  #[inline]
  pub fn update_model<M: Model>(&mut self, id: u32, model: M, clipped: bool) {
    self.get_model_sync(clipped).update_model(id, model);
  }

  #[inline]
  pub fn remove_model<M: Model>(&mut self, id: u32, clipped: bool) -> M {
    self.get_model_sync(clipped).remove_model(id)
  }

  #[inline]
  pub fn bulk_add_models<M: Model>(&mut self, models: Box<[M]>, clipped: bool) -> Box<[u32]> {
    self.get_model_sync(clipped).bulk_add_models(models)
  }

  #[inline]
  pub fn bulk_update_models<M: Model>(&mut self, ids: &[u32], models: Box<[M]>, clipped: bool) {
    self.get_model_sync(clipped).bulk_update_models(ids, models);
  }

  #[inline]
  pub fn bulk_remove_models<M: Model + Clone>(&mut self, ids: &[u32], clipped: bool) -> Box<[M]> {
    self.get_model_sync(clipped).bulk_remove_models(ids)
  }

  #[inline]
  pub fn add_text(&mut self, text: &Text, clipped: bool) -> TextId {
    self.get_glyph_renderer_mut().add_text(text, clipped)
  }

  #[inline]
  pub fn remove_text(&mut self, text_id: TextId) {
    self.get_glyph_renderer_mut().remove_text(text_id);
  }

  #[inline]
  pub fn add_icon(&mut self, icon: Icon, clipped: bool) -> IconId {
    self.get_glyph_renderer_mut().add_icon(icon, clipped)
  }

  #[inline]
  pub fn remove_icon(&mut self, icon_id: IconId) {
    self.get_glyph_renderer_mut().remove_icon(icon_id);
  }

  #[must_use]
  #[inline]
  pub fn get_text_size(&self, text_id: &TextId) -> (f32, f32) {
    self.get_glyph_renderer().get_text_size(text_id)
  }

  #[must_use]
  #[inline]
  pub fn get_icon_size(&self, icon_id: &IconId) -> (f32, f32) {
    self.get_glyph_renderer().get_icon_size(icon_id)
  }
}
