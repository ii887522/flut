use crate::{
  collections::{SparseSet, sparse_set},
  models::{Icon, Model, Text},
  renderers::{
    Renderer,
    renderer::{Created, FinishError},
  },
};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::mem;

const MIN_SEQ_LEN: usize = 1024;

#[derive(Clone, Copy)]
pub struct Id(sparse_set::Id);

#[derive(Clone, Copy)]
pub(crate) struct ModelId {
  z: u8,
  id: sparse_set::Id,
}

pub struct RendererRef<'render> {
  renderer: &'render mut Result<Renderer<Created>, FinishError>,
  model_ids: &'render mut SparseSet<ModelId>,
}

impl<'render> RendererRef<'render> {
  #[inline]
  pub(crate) const fn new(
    renderer: &'render mut Result<Renderer<Created>, FinishError>,
    model_ids: &'render mut SparseSet<ModelId>,
  ) -> Self {
    Self {
      renderer,
      model_ids,
    }
  }

  pub fn set_cam_position(&mut self, cam_position: Option<(f32, f32)>) {
    match self.renderer {
      Ok(renderer) => renderer.set_cam_position(cam_position),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.set_cam_position(cam_position);
      }
    }
  }

  pub fn set_cam_size(&mut self, cam_size: Option<(f32, f32)>) {
    match self.renderer {
      Ok(renderer) => renderer.set_cam_size(cam_size),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.set_cam_size(cam_size);
      }
    }
  }

  #[allow(private_bounds)]
  pub fn add_model<M: Model>(&mut self, model: M) -> Id {
    let model_renderers = M::get_renderers_mut(self.renderer);

    let add_resp = self.model_ids.add(ModelId {
      z: model.get_z(),
      id: model_renderers[model.get_z() as usize].add_model(model.into_pipeline_model()),
    });

    Id(add_resp.id)
  }

  #[allow(private_bounds)]
  pub fn update_model<M: Model>(&mut self, id: Id, model: M) {
    let model_renderers = M::get_renderers_mut(self.renderer);
    let model_id = self.model_ids.get_mut(id.0).unwrap();
    let z = model.get_z();

    if z == model_id.z {
      model_renderers[model_id.z as usize].update_model(model_id.id, model.into_pipeline_model());
      return;
    }

    let new_id = model_renderers[model_id.z as usize].add_model(model.into_pipeline_model());
    let old_model_id = mem::replace(model_id, ModelId { z, id: new_id });
    model_renderers[old_model_id.z as usize].remove_model(old_model_id.id);
  }

  #[allow(private_bounds)]
  pub fn remove_model<M: Model>(&mut self, id: Id) {
    let model_renderers = M::get_renderers_mut(self.renderer);
    let remove_resp = self.model_ids.remove(id.0);
    let model_id = remove_resp.item;
    model_renderers[model_id.z as usize].remove_model(model_id.id);
  }

  #[allow(private_bounds)]
  pub fn bulk_add_models<M: Model>(&mut self, models: Box<[M]>) -> Box<[Id]> {
    let model_renderers = M::get_renderers_mut(self.renderer);

    let z_to_models = models.into_iter().fold(
      FxHashMap::<u8, Vec<_>>::default(),
      |mut z_to_models, model| {
        z_to_models.entry(model.get_z()).or_default().push(model);
        z_to_models
      },
    );

    let model_ids = z_to_models
      .into_iter()
      .flat_map(|(z, models)| {
        model_renderers[z as usize]
          .bulk_add_models(
            models
              .into_par_iter()
              .with_min_len(MIN_SEQ_LEN)
              .map(|model| model.into_pipeline_model())
              .collect(),
          )
          .into_par_iter()
          .with_min_len(MIN_SEQ_LEN)
          .map(|id| ModelId { z, id })
          .collect::<Box<_>>()
      })
      .collect::<Box<_>>();

    let bulk_add_resp = self.model_ids.bulk_add(model_ids);

    bulk_add_resp
      .ids
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(Id)
      .collect()
  }

  #[allow(private_bounds)]
  pub fn bulk_update_models<M: Model>(&mut self, updates: Box<[(Id, M)]>) {
    let model_renderers = M::get_renderers_mut(self.renderer);

    let (same_z_updates, diff_z_updates): (Vec<_>, Vec<_>) = updates
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .partition(|(id, model)| {
        let model_id = self.model_ids.get(id.0).unwrap();
        model.get_z() == model_id.z
      });

    let z_to_same_z_updates = same_z_updates.into_iter().fold(
      FxHashMap::<u8, Vec<_>>::default(),
      |mut z_to_updates, (id, model)| {
        z_to_updates
          .entry(model.get_z())
          .or_default()
          .push((id, model));

        z_to_updates
      },
    );

    for (z, updates) in z_to_same_z_updates {
      model_renderers[z as usize].bulk_update_models(
        updates
          .into_par_iter()
          .with_min_len(MIN_SEQ_LEN)
          .map(|(id, model)| (id.0, model.into_pipeline_model()))
          .collect::<Box<_>>(),
      );
    }

    let new_z_to_diff_z_updates = diff_z_updates.iter().fold(
      FxHashMap::<u8, Vec<_>>::default(),
      |mut z_to_updates, (id, model)| {
        z_to_updates
          .entry(model.get_z())
          .or_default()
          .push((*id, model.clone()));

        z_to_updates
      },
    );

    let old_z_to_diff_z_updates = diff_z_updates.iter().fold(
      FxHashMap::<u8, Vec<_>>::default(),
      |mut z_to_updates, (id, model)| {
        let model_id = self.model_ids.get(id.0).unwrap();

        z_to_updates
          .entry(model_id.z)
          .or_default()
          .push((*id, model.clone()));

        z_to_updates
      },
    );

    let model_id_updates = new_z_to_diff_z_updates
      .into_iter()
      .flat_map(|(z, updates)| {
        let new_ids = model_renderers[z as usize].bulk_add_models(
          updates
            .par_iter()
            .with_min_len(MIN_SEQ_LEN)
            .map(|(_, model)| model.clone().into_pipeline_model())
            .collect::<Box<_>>(),
        );

        updates
          .into_par_iter()
          .with_min_len(MIN_SEQ_LEN)
          .zip(new_ids.into_par_iter().with_min_len(MIN_SEQ_LEN))
          .map(|((id, model), new_id)| {
            (
              id.0,
              ModelId {
                z: model.get_z(),
                id: new_id,
              },
            )
          })
          .collect::<Box<_>>()
      })
      .collect::<Box<_>>();

    for (z, updates) in old_z_to_diff_z_updates {
      let old_model_ids = updates
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(|(id, _)| {
          let model_id = self.model_ids.get(id.0).unwrap();
          model_id.id
        })
        .collect::<Box<_>>();

      model_renderers[z as usize].bulk_remove_models(&old_model_ids);
    }

    self.model_ids.bulk_update(model_id_updates);
  }

  #[allow(private_bounds)]
  pub fn bulk_remove_models<M: Model>(&mut self, ids: &[Id]) {
    let model_renderers = M::get_renderers_mut(self.renderer);

    let z_to_ids = ids
      .iter()
      .map(|&id| self.model_ids.get(id.0).unwrap())
      .fold(
        FxHashMap::<u8, Vec<_>>::default(),
        |mut z_to_ids, &model_id| {
          z_to_ids.entry(model_id.z).or_default().push(model_id.id);
          z_to_ids
        },
      );

    let model_ids = ids.into_par_iter().map(|id| id.0).collect::<Box<_>>();
    self.model_ids.bulk_remove(&model_ids);

    for (z, ids) in z_to_ids {
      model_renderers[z as usize].bulk_remove_models(&ids);
    }
  }

  pub fn add_text(&mut self, text: Text) -> Id {
    match self.renderer {
      Ok(renderer) => Id(renderer.get_text_renderer().add_text(text, false)),
      Err(FinishError::WindowMinimized(renderer)) => {
        Id(renderer.get_text_renderer().add_text(text, false))
      }
    }
  }

  pub fn update_text(&mut self, id: Id, text: Text) {
    match self.renderer {
      Ok(renderer) => renderer.get_text_renderer().update_text(id.0, text, false),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_text_renderer().update_text(id.0, text, false)
      }
    }
  }

  pub fn remove_text(&mut self, id: Id) {
    match self.renderer {
      Ok(renderer) => renderer.get_text_renderer().remove_text(id.0),
      Err(FinishError::WindowMinimized(renderer)) => renderer.get_text_renderer().remove_text(id.0),
    }
  }

  pub fn bulk_add_texts(&mut self, texts: Box<[Text]>) -> Box<[Id]> {
    match self.renderer {
      Ok(renderer) => renderer
        .get_text_renderer()
        .bulk_add_text(texts, false)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .bulk_add_text(texts, false)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
    }
  }

  pub fn bulk_update_texts(&mut self, updates: Box<[(Id, Text)]>) {
    let updates = updates
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|(id, text)| (id.0, text))
      .collect();

    match self.renderer {
      Ok(renderer) => renderer
        .get_text_renderer()
        .bulk_update_text(updates, false),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .bulk_update_text(updates, false),
    }
  }

  pub fn bulk_remove_texts(&mut self, ids: &[Id]) {
    let ids = ids
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|id| id.0)
      .collect::<Box<_>>();

    match self.renderer {
      Ok(renderer) => renderer.get_text_renderer().bulk_remove_text(&ids),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_text_renderer().bulk_remove_text(&ids)
      }
    }
  }

  pub fn add_icon(&mut self, icon: Icon) -> Id {
    match self.renderer {
      Ok(renderer) => Id(renderer.get_text_renderer().add_text(icon.into(), true)),
      Err(FinishError::WindowMinimized(renderer)) => {
        Id(renderer.get_text_renderer().add_text(icon.into(), true))
      }
    }
  }

  pub fn update_icon(&mut self, id: Id, icon: Icon) {
    match self.renderer {
      Ok(renderer) => renderer
        .get_text_renderer()
        .update_text(id.0, icon.into(), true),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer
          .get_text_renderer()
          .update_text(id.0, icon.into(), true)
      }
    }
  }

  pub fn remove_icon(&mut self, id: Id) {
    match self.renderer {
      Ok(renderer) => renderer.get_text_renderer().remove_text(id.0),
      Err(FinishError::WindowMinimized(renderer)) => renderer.get_text_renderer().remove_text(id.0),
    }
  }

  pub fn bulk_add_icons(&mut self, icons: Box<[Icon]>) -> Box<[Id]> {
    let icon_texts = icons
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|icon| icon.into())
      .collect();

    match self.renderer {
      Ok(renderer) => renderer
        .get_text_renderer()
        .bulk_add_text(icon_texts, true)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
      Err(FinishError::WindowMinimized(renderer)) => renderer
        .get_text_renderer()
        .bulk_add_text(icon_texts, true)
        .into_par_iter()
        .with_min_len(MIN_SEQ_LEN)
        .map(Id)
        .collect(),
    }
  }

  pub fn bulk_update_icons(&mut self, updates: Box<[(Id, Icon)]>) {
    let updates = updates
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|(id, icon)| (id.0, icon.into()))
      .collect();

    match self.renderer {
      Ok(renderer) => renderer.get_text_renderer().bulk_update_text(updates, true),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_text_renderer().bulk_update_text(updates, true)
      }
    }
  }

  pub fn bulk_remove_icons(&mut self, ids: &[Id]) {
    let ids = ids
      .into_par_iter()
      .with_min_len(MIN_SEQ_LEN)
      .map(|id| id.0)
      .collect::<Box<_>>();

    match self.renderer {
      Ok(renderer) => renderer.get_text_renderer().bulk_remove_text(&ids),
      Err(FinishError::WindowMinimized(renderer)) => {
        renderer.get_text_renderer().bulk_remove_text(&ids)
      }
    }
  }
}
