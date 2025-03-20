pub(super) mod basic {
  vulkano_shaders::shader! {
    shaders: {
      vs: { ty: "vertex", path: "glsl/basic.vert" },
      fs: { ty: "fragment", path: "glsl/basic.frag" }
    }
  }
}
