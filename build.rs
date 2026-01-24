// region: Clippy lints
//
// Category lints
#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::nursery)]
#![deny(clippy::pedantic)]
//
// Individual lints
#![deny(clippy::absolute_paths)]
#![deny(clippy::alloc_instead_of_core)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::as_pointer_underscore)]
#![deny(clippy::as_underscore)]
#![deny(clippy::assertions_on_result_states)]
#![deny(clippy::big_endian_bytes)]
#![deny(clippy::cfg_not_test)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::create_dir)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::decimal_literal_representation)]
#![deny(clippy::default_numeric_fallback)]
#![deny(clippy::default_union_representation)]
#![deny(clippy::deref_by_slicing)]
#![deny(clippy::disallowed_script_idents)]
#![deny(clippy::doc_include_without_cfg)]
#![deny(clippy::else_if_without_else)]
#![deny(clippy::empty_drop)]
#![deny(clippy::empty_enum_variants_with_brackets)]
#![deny(clippy::empty_structs_with_brackets)]
#![deny(clippy::error_impl_error)]
#![deny(clippy::exit)]
#![deny(clippy::expect_used)]
#![deny(clippy::field_scoped_visibility_modifiers)]
#![deny(clippy::filetype_is_file)]
#![deny(clippy::float_cmp_const)]
#![deny(clippy::fn_to_numeric_cast_any)]
#![deny(clippy::get_unwrap)]
#![deny(clippy::host_endian_bytes)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::impl_trait_in_params)]
#![deny(clippy::infinite_loop)]
#![deny(clippy::inline_asm_x86_att_syntax)]
#![deny(clippy::inline_asm_x86_intel_syntax)]
#![deny(clippy::integer_division)]
#![deny(clippy::iter_over_hash_type)]
#![deny(clippy::large_include_file)]
#![deny(clippy::let_underscore_must_use)]
#![deny(clippy::let_underscore_untyped)]
#![deny(clippy::little_endian_bytes)]
#![deny(clippy::lossy_float_literal)]
#![deny(clippy::map_err_ignore)]
#![deny(clippy::map_with_unused_argument_over_ranges)]
#![deny(clippy::mem_forget)]
#![deny(clippy::min_ident_chars)]
#![deny(clippy::missing_asserts_for_indexing)]
#![deny(clippy::mixed_read_write_in_expression)]
#![deny(clippy::module_name_repetitions)]
#![deny(clippy::multiple_inherent_impl)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::mutex_atomic)]
#![deny(clippy::mutex_integer)]
#![deny(clippy::needless_raw_strings)]
#![deny(clippy::non_zero_suggestions)]
#![deny(clippy::partial_pub_fields)]
#![deny(clippy::pathbuf_init_then_push)]
#![deny(clippy::pattern_type_mismatch)]
#![deny(clippy::pointer_format)]
#![deny(clippy::precedence_bits)]
#![deny(clippy::print_stdout)]
#![deny(clippy::pub_use)]
#![deny(clippy::pub_without_shorthand)]
#![deny(clippy::rc_buffer)]
#![deny(clippy::rc_mutex)]
#![deny(clippy::redundant_test_prefix)]
#![deny(clippy::redundant_type_annotations)]
#![deny(clippy::ref_patterns)]
#![deny(clippy::renamed_function_params)]
#![deny(clippy::rest_pat_in_fully_bound_structs)]
#![deny(clippy::return_and_then)]
#![deny(clippy::same_name_method)]
#![deny(clippy::self_named_module_files)]
#![deny(clippy::semicolon_inside_block)]
#![deny(clippy::single_char_lifetime_names)]
#![deny(clippy::str_to_string)]
#![deny(clippy::string_lit_chars_any)]
#![deny(clippy::string_slice)]
#![deny(clippy::suspicious_xor_used_as_pow)]
#![deny(clippy::todo)]
#![deny(clippy::try_err)]
#![deny(clippy::unimplemented)]
#![deny(clippy::unnecessary_safety_comment)]
#![deny(clippy::unnecessary_safety_doc)]
#![deny(clippy::unnecessary_self_imports)]
#![deny(clippy::unreachable)]
#![deny(clippy::unseparated_literal_suffix)]
#![deny(clippy::unused_result_ok)]
#![deny(clippy::unused_trait_names)]
#![deny(clippy::use_debug)]
#![deny(clippy::verbose_file_reads)]
//
// Allowed lints
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::result_large_err)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
//
// endregion

use shaderc::{CompileOptions, Compiler, EnvVersion, OptimizationLevel, ShaderKind, TargetEnv};
use std::{env, fs, path::Path};

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();
  let shader_dir = Path::new("src/shaders");
  let shader_compiler = Compiler::new().unwrap();
  let mut shader_compile_options = CompileOptions::new().unwrap();
  shader_compile_options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_3 as u32);
  shader_compile_options.set_optimization_level(OptimizationLevel::Performance);
  shader_compile_options.set_warnings_as_errors();

  compile_shaders(
    &shader_compiler,
    &shader_compile_options,
    shader_dir,
    &out_dir,
  );
}

fn compile_shaders(compiler: &Compiler, options: &CompileOptions, dir: &Path, out_dir: &str) {
  let dir = fs::read_dir(dir).unwrap();

  for entry in dir {
    let entry = entry.unwrap();
    let path = entry.path();

    if path.is_dir() {
      compile_shaders(compiler, options, &path, out_dir);
    } else if let Some(shader_kind) = get_shader_kind(&path) {
      compile_shader(compiler, options, &path, shader_kind, out_dir);
    } else {
      // Skip non-shader files
    }
  }
}

fn get_shader_kind(path: &Path) -> Option<ShaderKind> {
  match path.extension()?.to_str()? {
    "vert" => Some(ShaderKind::Vertex),
    "frag" => Some(ShaderKind::Fragment),
    "comp" => Some(ShaderKind::Compute),
    "geom" => Some(ShaderKind::Geometry),
    "tesc" => Some(ShaderKind::TessControl),
    "tese" => Some(ShaderKind::TessEvaluation),
    _ => None,
  }
}

fn compile_shader(
  compiler: &Compiler,
  options: &CompileOptions,
  path: &Path,
  kind: ShaderKind,
  out_dir: &str,
) {
  println!("cargo:rerun-if-changed={}", path.display());
  let source = fs::read_to_string(path).unwrap();
  let filename = path.file_name().unwrap().to_string_lossy();

  let binary = compiler
    .compile_into_spirv(&source, kind, &filename, "main", Some(options))
    .unwrap();

  let out_path = Path::new(out_dir).join(format!("{filename}.spv"));
  fs::write(&out_path, binary.as_binary_u8()).unwrap();
}
