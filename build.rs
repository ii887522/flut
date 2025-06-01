use shaderc::{OptimizationLevel, ShaderKind};
use std::fs;

fn main() {
  // Tell the build script to only run again if source shaders changed
  println!("cargo:rerun-if-changed=glsl");

  const OUT_DIR_PATH: &str = "target/shaders";

  // Create destination path if necessary
  fs::create_dir_all(OUT_DIR_PATH).unwrap();

  let compiler = shaderc::Compiler::new().unwrap();
  let mut compile_options = shaderc::CompileOptions::new().unwrap();
  compile_options.set_optimization_level(OptimizationLevel::Performance);

  for glsl_file in fs::read_dir("glsl").unwrap() {
    let glsl_file = glsl_file.unwrap();
    let glsl_file_path = glsl_file.path();
    let glsl_code = fs::read_to_string(&glsl_file_path).unwrap();
    let glsl_file_name = glsl_file.file_name();
    let glsl_file_name = glsl_file_name.to_str().unwrap();
    let glsl_file_ext = glsl_file_path.extension().unwrap();
    let glsl_file_ext = glsl_file_ext.to_str().unwrap();

    let shader_kind = match glsl_file_ext {
      "frag" => ShaderKind::Fragment,
      "vert" => ShaderKind::Vertex,
      file_ext => unimplemented!("Unknown shader type: {}", file_ext),
    };

    let spirv_code = compiler
      .compile_into_spirv(
        &glsl_code,
        shader_kind,
        glsl_file_name,
        "main",
        Some(&compile_options),
      )
      .unwrap();

    fs::write(
      format!("{OUT_DIR_PATH}/{glsl_file_name}.spv"),
      spirv_code.as_binary_u8(),
    )
    .unwrap();
  }
}
