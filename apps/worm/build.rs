use std::{env, process::Command};

fn main() {
  let profile = env::var("PROFILE").unwrap();
  let profile = if profile == "debug" { "dev" } else { "release" };

  // Build dylib for hot reloading
  let status = Command::new("cargo")
    .current_dir("../..")
    .args([
      "rustc",
      "--package",
      "worm_lib",
      "--target-dir",
      "apps/worm/lib/target",
      "--crate-type",
      "dylib",
      "--features",
      "reload",
      "--profile",
      profile,
    ])
    .status()
    .unwrap();

  if !status.success() {
    panic!("Failed to build worm_lib as dylib");
  }

  // Tell cargo to rerun this build script if any library files change
  println!("cargo::rerun-if-changed=lib/src/");
  println!("cargo::rerun-if-changed=lib/Cargo.toml");
}
