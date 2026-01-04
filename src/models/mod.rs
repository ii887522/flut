pub mod align;
pub mod atlas_sizes;
pub mod icon;
pub mod model_capacities;
pub mod rect;
pub mod round_rect;
pub mod text;
pub mod write;

pub use align::Align;
pub use atlas_sizes::AtlasSizes;
pub use icon::Icon;
pub use model_capacities::ModelCapacities;
pub use rect::Rect;
pub use round_rect::RoundRect;
pub use text::Text;
pub use write::Write;

use crate::{
  pipelines,
  renderers::{
    ModelRenderer, Renderer,
    renderer::{self, FinishError},
  },
};

pub(super) trait Model: Clone + Send + Sync {
  type PipelineModel: pipelines::Model;

  fn get_renderers_mut(
    renderer: &mut Result<Renderer<renderer::Created>, FinishError>,
  ) -> &mut [ModelRenderer<Self::PipelineModel>];

  fn get_z(&self) -> u8;
  fn into_pipeline_model(self) -> Self::PipelineModel;
}
