use crate::{
  models::Model,
  renderer::{Created, Creating, Renderer},
};

pub struct RendererRef<'render>(&'render mut Result<Renderer<Created>, Renderer<Creating>>);

impl<'render> RendererRef<'render> {
  #[inline]
  pub(super) const fn new(
    renderer: &'render mut Result<Renderer<Created>, Renderer<Creating>>,
  ) -> Self {
    Self(renderer)
  }

  pub fn add_model<M: Model>(&mut self, model: M) -> u32 {
    let model_sync = match *self.0 {
      Ok(ref mut renderer) => M::get_sync(renderer),
      Err(ref mut renderer) => M::get_sync(renderer),
    };

    model_sync.add_model(model)
  }

  pub fn update_model<M: Model>(&mut self, id: u32, model: M) {
    let model_sync = match *self.0 {
      Ok(ref mut renderer) => M::get_sync(renderer),
      Err(ref mut renderer) => M::get_sync(renderer),
    };

    model_sync.update_model(id, model);
  }

  pub fn remove_model<M: Model>(&mut self, id: u32) {
    let model_sync = match *self.0 {
      Ok(ref mut renderer) => M::get_sync(renderer),
      Err(ref mut renderer) => M::get_sync(renderer),
    };

    model_sync.remove_model(id);
  }

  pub fn bulk_add_models<M: Model>(&mut self, models: Box<[M]>) -> Box<[u32]> {
    let model_sync = match *self.0 {
      Ok(ref mut renderer) => M::get_sync(renderer),
      Err(ref mut renderer) => M::get_sync(renderer),
    };

    model_sync.bulk_add_models(models)
  }

  pub fn bulk_update_models<M: Model>(&mut self, ids: &[u32], models: Box<[M]>) {
    let model_sync = match *self.0 {
      Ok(ref mut renderer) => M::get_sync(renderer),
      Err(ref mut renderer) => M::get_sync(renderer),
    };

    model_sync.bulk_update_models(ids, models);
  }

  pub fn bulk_remove_models<M: Model + Clone>(&mut self, ids: &[u32]) -> Box<[M]> {
    let model_sync = match *self.0 {
      Ok(ref mut renderer) => M::get_sync(renderer),
      Err(ref mut renderer) => M::get_sync(renderer),
    };

    model_sync.bulk_remove_models(ids)
  }
}
