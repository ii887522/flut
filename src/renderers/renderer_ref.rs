use crate::{
  collections::sparse_set,
  models::{Rect, Text},
  renderers::{
    Renderer,
    renderer::{Created, FinishError},
  },
};
use rayon::prelude::*;

const MIN_SEQ_LEN: usize = 1024;

#[derive(Clone, Copy)]
pub struct Id(sparse_set::Id);

pub struct RendererRef<'render>(&'render mut Result<Renderer<Created>, FinishError>);

impl<'render> RendererRef<'render> {
  #[inline]
  pub(crate) const fn new(renderer: &'render mut Result<Renderer<Created>, FinishError>) -> Self {
    Self(renderer)
  }

  pub fn set_cam_position(&mut self, cam_position: Option<(f32, f32)>) {
    match self.0 {
      Ok(renderer) => renderer.set_cam_position(cam_position),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.set_cam_position(cam_position);
      }
    }
  }

  pub fn set_cam_size(&mut self, cam_size: Option<(f32, f32)>) {
    match self.0 {
      Ok(renderer) => renderer.set_cam_size(cam_size),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.set_cam_size(cam_size);
      }
    }
  }

  pub fn add_rect(&mut self, rect: Rect) -> Id {
    match self.0 {
      Ok(renderer) => Id(
        renderer
          .get_text_renderer()
          .get_glyph_renderer_mut()
          .add_model(rect.into()),
      ),
      Err(FinishError::WindowMinimized(renderer)) => Id(
        renderer
          .get_text_renderer()
          .get_glyph_renderer_mut()
          .add_model(rect.into()),
      ),
    }
  }

  pub fn update_rect(&mut self, id: Id, rect: Rect) {
    match self.0 {
      Ok(renderer) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .update_model(id.0, rect.into()),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .update_model(id.0, rect.into()),
    }
  }

  pub fn remove_rect(&mut self, id: Id) {
    match self.0 {
      Ok(renderer) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .remove_model(id.0),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .remove_model(id.0),
    }
  }

  pub fn bulk_add_rects(&mut self, rects: Box<[Rect]>) -> Box<[Id]> {
    let rects = rects
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|rect| rect.into())
      .collect();

    match self.0 {
      Ok(renderer) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .bulk_add_models(rects)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .bulk_add_models(rects)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
    }
  }

  pub fn bulk_update_rects(&mut self, updates: Box<[(Id, Rect)]>) {
    let updates = updates
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|(id, rect)| (id.0, rect.into()))
      .collect();

    match self.0 {
      Ok(renderer) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .bulk_update_models(updates),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .bulk_update_models(updates),
    }
  }

  pub fn bulk_remove_rects(&mut self, ids: &[Id]) {
    let ids = ids
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|id| id.0)
      .collect::<Box<_>>();

    match self.0 {
      Ok(renderer) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .bulk_remove_models(&ids),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .get_glyph_renderer_mut()
        .bulk_remove_models(&ids),
    }
  }

  pub fn add_text(&mut self, text: Text) -> Id {
    match self.0 {
      Ok(renderer) => Id(renderer.get_text_renderer().add_text(text)),
      Err(FinishError::WindowMinimized(renderer)) => {
        Id(renderer.get_text_renderer().add_text(text))
      }
    }
  }

  pub fn update_text(&mut self, id: Id, text: Text) {
    match self.0 {
      Ok(renderer) => renderer.get_text_renderer().update_text(id.0, text),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_text_renderer().update_text(id.0, text)
      }
    }
  }

  pub fn remove_text(&mut self, id: Id) {
    match self.0 {
      Ok(renderer) => renderer.get_text_renderer().remove_text(id.0),
      Err(FinishError::WindowMinimized(renderer)) => renderer.get_text_renderer().remove_text(id.0),
    }
  }

  pub fn bulk_add_text(&mut self, texts: Box<[Text]>) -> Box<[Id]> {
    match self.0 {
      Ok(renderer) => renderer
        .get_text_renderer()
        .bulk_add_text(texts)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .bulk_add_text(texts)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
    }
  }

  pub fn bulk_update_text(&mut self, updates: Box<[(Id, Text)]>) {
    let updates = updates
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|(id, text)| (id.0, text))
      .collect();

    match self.0 {
      Ok(renderer) => renderer.get_text_renderer().bulk_update_text(updates),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_text_renderer().bulk_update_text(updates)
      }
    }
  }

  pub fn bulk_remove_text(&mut self, ids: &[Id]) {
    let ids = ids
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|id| id.0)
      .collect::<Box<_>>();

    match self.0 {
      Ok(renderer) => renderer.get_text_renderer().bulk_remove_text(&ids),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_text_renderer().bulk_remove_text(&ids)
      }
    }
  }
}
