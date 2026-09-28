use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf, process::Command};
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../..")
    .canonicalize()?;
  let mut baseline: Value =
    serde_json::from_slice(&fs::read(root.join("docs/dev-baseline.json"))?)?;
  let args: Vec<_> = std::env::args().skip(1).collect();
  match args.as_slice() {
    [] => {}
    [flag, path] if flag == "--dev" => {
      let lock: toml::Value =
        toml::from_str(&fs::read_to_string(PathBuf::from(path).join("Cargo.lock"))?)?;
      let packages = lock["package"]
        .as_array()
        .ok_or("invalid dev lock")?
        .iter()
        .filter(|p| p.get("source").is_some())
        .collect::<Vec<_>>();
      baseline["packages"] = serde_json::to_value(packages)?;
    }
    _ => return Err("usage: xtask [--dev <dev-worktree>]".into()),
  }
  let expected: HashSet<_> = baseline["packages"]
    .as_array()
    .ok_or("missing baseline packages")?
    .iter()
    .map(|p| {
      (
        p["name"].as_str().unwrap(),
        p["version"].as_str().unwrap(),
        p["source"].as_str().unwrap(),
      )
    })
    .collect();
  let output = Command::new("cargo")
    .args(["metadata", "--format-version", "1", "--locked", "--offline"])
    .current_dir(&root)
    .output()?;
  if !output.status.success() {
    return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
  }
  let metadata: Value = serde_json::from_slice(&output.stdout)?;
  let mut checked = 0;
  for package in metadata["packages"]
    .as_array()
    .ok_or("missing metadata packages")?
  {
    if let Some(source) = package["source"].as_str() {
      let key = (
        package["name"].as_str().unwrap(),
        package["version"].as_str().unwrap(),
        source,
      );
      if !expected.contains(&key) {
        return Err(format!("dependency drift from dev baseline: {key:?}").into());
      }
      checked += 1;
    } else {
      let manifest = package["manifest_path"]
        .as_str()
        .ok_or("no manifest path")?;
      let contents = fs::read_to_string(manifest)?;
      // Experiments inherit every dependency; only the workspace root declares versions/features.
      let mut dependencies = false;
      for line in contents.lines().map(str::trim) {
        if line.starts_with('[') {
          dependencies = line.contains("dependencies]");
          continue;
        }
        if dependencies
          && !line.is_empty()
          && !line.starts_with('#')
          && !line.ends_with(".workspace = true")
        {
          return Err(format!("{manifest}: dependency must inherit workspace: {line}").into());
        }
      }
    }
  }
  let rustc = Command::new("rustc").arg("-Vv").output()?;
  if !rustc.status.success() {
    return Err("rustc failed".into());
  }
  let current = String::from_utf8(rustc.stdout)?;
  let expected_rustc = baseline["rustc"].as_str().ok_or("missing rustc baseline")?;
  // Host may differ on another desktop OS; compiler release and commit must match.
  for prefix in ["release:", "commit-hash:"] {
    if current.lines().find(|l| l.starts_with(prefix))
      != expected_rustc.lines().find(|l| l.starts_with(prefix))
    {
      return Err(format!("compiler differs from dev baseline: {prefix}").into());
    }
  }
  println!(
    "Verified {checked} registry packages against dev working-tree baseline, inherited dependencies and rustc release/commit."
  );
  Ok(())
}
