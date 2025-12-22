use flut::renderers::renderer_ref;

pub(crate) struct Countdown {
  pub(crate) countdown: u32,
  pub(crate) render_id: renderer_ref::Id,
  pub(crate) accum: f32,
}
