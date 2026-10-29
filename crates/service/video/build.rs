//! Link the Windows GNU C++ runtime statically for portable video consumers.

use std::{env, path::PathBuf};

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  if env::var("TARGET").as_deref() != Ok("x86_64-pc-windows-gnu") {
    return;
  }

  let compiler = cc::Build::new().cpp(true).get_compiler();
  let output = compiler
    .to_command()
    .arg("-print-file-name=libstdc++.a")
    .output()
    .expect("query the Windows GNU compiler's static C++ runtime");
  assert!(output.status.success(), "static C++ runtime lookup failed");
  let archive = PathBuf::from(
    String::from_utf8(output.stdout)
      .expect("compiler runtime path must be UTF-8")
      .trim(),
  );
  assert!(
    archive.is_absolute() && archive.is_file(),
    "the selected GNU compiler must provide libstdc++.a: {}",
    archive.display()
  );
  println!(
    "cargo:rustc-link-search=native={}",
    archive
      .parent()
      .expect("runtime archive directory")
      .display()
  );
  // OpenH264 is a transitive dependency, so preserve runtime objects until its symbols resolve.
  println!("cargo:rustc-link-lib=static:+whole-archive,-bundle=stdc++");
}
