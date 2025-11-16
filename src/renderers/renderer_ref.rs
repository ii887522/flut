use crate::{
  collections::sparse_set,
  models::Rect,
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

  pub fn add_rect(&mut self, rect: Rect) -> Id {
    match self.0 {
      Ok(renderer) => Id(renderer.get_rect_renderer().add_model(rect.into())),
      Err(FinishError::WindowMinimized(renderer)) => {
        Id(renderer.get_rect_renderer().add_model(rect.into()))
      }
    }
  }

  pub fn update_rect(&mut self, id: Id, rect: Rect) {
    match self.0 {
      Ok(renderer) => renderer.get_rect_renderer().update_model(id.0, rect.into()),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_rect_renderer().update_model(id.0, rect.into())
      }
    }
  }

  pub fn remove_rect(&mut self, id: Id) {
    match self.0 {
      Ok(renderer) => renderer.get_rect_renderer().remove_model(id.0),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_rect_renderer().remove_model(id.0)
      }
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
        .get_rect_renderer()
        .bulk_add_models(rects)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_rect_renderer()
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
      Ok(renderer) => renderer.get_rect_renderer().bulk_update_models(updates),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_rect_renderer().bulk_update_models(updates)
      }
    }
  }

  pub fn bulk_remove_rects(&mut self, ids: &[Id]) {
    let ids = ids
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|id| id.0)
      .collect::<Vec<_>>();

    let ids = ids.as_slice();

    match self.0 {
      Ok(renderer) => renderer.get_rect_renderer().bulk_remove_models(ids),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_rect_renderer().bulk_remove_models(ids)
      }
    }
  }
}
