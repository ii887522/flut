use crate::pipelines::{CreatedPipeline, CreatingPipeline};

pub(crate) trait Model {
  type PushConsts;
  type CreatingPipeline: CreatingPipeline<Model = Self>;
  type CreatedPipeline: CreatedPipeline<Model = Self>;
}
