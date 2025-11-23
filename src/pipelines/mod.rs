pub(super) mod model;
pub(super) mod rect_pipeline;

pub(super) use model::Model;

use ash::vk;

pub(super) trait CreatingPipeline {
  type Model: Model;

  fn new(device: &ash::Device) -> Self;

  fn finish(
    self,
    device: &ash::Device,
    render_pass: vk::RenderPass,
    cache: vk::PipelineCache,
    swapchain_image_extent: vk::Extent2D,
  ) -> <Self::Model as Model>::CreatedPipeline;

  fn drop(self, device: &ash::Device);
}

pub(super) trait CreatedPipeline {
  type Model: Model;

  fn get_pipeline(&self) -> vk::Pipeline;
  fn get_pipeline_layout(&self) -> vk::PipelineLayout;
  fn on_swapchain_suboptimal(self) -> <Self::Model as Model>::CreatingPipeline;
  fn drop(self, device: &ash::Device);
}
