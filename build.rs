use shaderc::{CompileOptions, EnvVersion, OptimizationLevel, ShaderKind, TargetEnv};
use std::{env, fs, path::Path};

fn main() {
  // Create the directory for compiled shaders
  let out_dir = env::var("OUT_DIR").unwrap();
  let spv_dir = Path::new(&out_dir).join("spv");
  fs::create_dir_all(&spv_dir).unwrap();

  let shader_compiler = shaderc::Compiler::new().unwrap();

  let mut compile_options = CompileOptions::new().unwrap();
  compile_options.set_optimization_level(OptimizationLevel::Performance);
  compile_options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_3 as _);
  compile_options.set_warnings_as_errors();

  // Compile all shaders in the src/shaders directory
  for entry in fs::read_dir("src/shaders").unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();

    if !path.is_file() {
      continue;
    }

    let Some(path_ext) = path.extension() else {
      println!("cargo::warning=Unrecognized file: {path:?} - skipping file");
      continue;
    };

    let shader_kind = match path_ext.to_str() {
      Some("vert") => Some(ShaderKind::Vertex),
      Some("frag") => Some(ShaderKind::Fragment),
      _ => None,
    };

    let Some(shader_kind) = shader_kind else {
      println!("cargo::warning=Unrecognized shader extension: {path_ext:?} - skipping file");
      continue;
    };

    let file_name = path.file_name().unwrap();
    let file_name = file_name.to_str().unwrap();
    let out_path = spv_dir.join(format!("{file_name}.spv"));
    let source_code = fs::read_to_string(&path).unwrap();

    let compiled = shader_compiler
      .compile_into_spirv(
        &source_code,
        shader_kind,
        path.to_str().unwrap(),
        "main",
        Some(&compile_options),
      )
      .unwrap();

    fs::write(&out_path, compiled.as_binary_u8()).unwrap();
  }

  // Tell cargo to rerun this build script if shaders change
  println!("cargo::rerun-if-changed=src/shaders/");
}
