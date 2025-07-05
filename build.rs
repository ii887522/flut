use shaderc::ShaderKind;
use std::fs;

fn main() {
  // Tell the build script to only run again if we change our source shaders
  println!("cargo:rerun-if-changed=glsl");

  const OUT_DIR_PATH: &str = "target/spv";

  // Create destination path if necessary
  fs::create_dir_all(OUT_DIR_PATH).unwrap();

  let shader_compiler = shaderc::Compiler::new().unwrap();
  let mut compile_options = shaderc::CompileOptions::new().unwrap();
  compile_options.set_optimization_level(shaderc::OptimizationLevel::Performance);
  compile_options.set_warnings_as_errors();

  for entry in fs::read_dir("glsl").unwrap() {
    let entry = entry.unwrap();
    let file_name = entry.file_name();
    let file_name = file_name.to_string_lossy();

    let shader_kind = match &*entry.path().extension().unwrap().to_string_lossy() {
      "frag" => ShaderKind::Fragment,
      "vert" => ShaderKind::Vertex,
      ext => panic!("Unknown shader extension: {ext}"),
    };

    let glsl_code = fs::read_to_string(entry.path()).unwrap();

    let spv_artifact = shader_compiler
      .compile_into_spirv(
        &glsl_code,
        shader_kind,
        &file_name,
        "main",
        Some(&compile_options),
      )
      .unwrap();

    fs::write(
      format!("{OUT_DIR_PATH}/{file_name}.spv"),
      spv_artifact.as_binary_u8(),
    )
    .unwrap();
  }
}
