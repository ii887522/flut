use crate::pipelines::{CreatedPipeline, CreatingPipeline};

pub(crate) trait Model {
  type PushConsts;
  type CreatingPipeline: CreatingPipeline<Model = Self>;
  type CreatedPipeline: CreatedPipeline<Model = Self>;

  fn get_name() -> &'static str;
  fn get_vertex_count() -> usize;
}
