mod model_renderer;
pub(super) mod renderer;
pub mod renderer_ref;
mod text_renderer;

use model_renderer::ModelRenderer;
pub(super) use renderer::Renderer;
pub use renderer_ref::RendererRef;

const MAX_IN_FLIGHT_FRAME_COUNT: usize = 2;
