//! Regression checks for documented Lua library members and independently executed API examples.

use std::{fs, path::PathBuf, time::Duration};

use tg_service_layout::Size;
use tg_service_lua::{LuaApiConfig, LuaService, LuaSessionKind, LuaSessionSpec};

struct Fixture(PathBuf);

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

#[test]
fn api_examples_execute_with_fixture_assets() {
  let root = std::env::temp_dir().join(format!("tg-doc-examples-{}", std::process::id()));
  fs::create_dir(&root).unwrap();
  let fixture = Fixture(root);
  fs::create_dir(fixture.0.join("scripts")).unwrap();
  fs::create_dir(fixture.0.join("assets")).unwrap();
  let entry = fixture.0.join("scripts/main.lua");
  let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dev_docs/zh_cn/api");
  // These fixtures enqueue asynchronous requests; no application loop executes their side
  // effects.

  fs::write(fixture.0.join("assets/file.txt"), "Hello Tui Game").unwrap();
  for directory in ["dir", "test", "c", "js", "rust/src", "ui"] {
    fs::create_dir_all(fixture.0.join("assets").join(directory)).unwrap();
  }
  for file in [
    "test/test.txt",
    "c/game.c",
    "c/main.c",
    "js/data.json",
    "js/main.js",
    "rust/src/main.rs",
    "rust/Cargo.toml",
  ] {
    fs::write(fixture.0.join("assets").join(file), "fixture").unwrap();
  }
  fs::write(fixture.0.join("scripts/value.lua"), "return \"ready\", nil").unwrap();
  image::RgbImage::new(2, 2)
    .save(fixture.0.join("assets/ui/title.png"))
    .unwrap();
  let libraries = [
    "align",
    "base",
    "char",
    "color",
    "date",
    "timer",
    "debug",
    "draw",
    "encoding",
    "events",
    "file",
    "game",
    "i18n",
    "image",
    "ime",
    "keyboard",
    "lifecycle",
    "loader",
    "math",
    "measurement",
    "random",
    "serialization",
    "slice",
    "string",
    "table",
    "utf8",
  ];
  let mut failures = Vec::new();
  let mut count = 0;
  for library in libraries {
    let markdown = fs::read_to_string(docs.join(format!("{library}.md"))).unwrap();
    let mut example_section = false;
    let mut in_example = false;
    let mut source = String::new();
    let mut start = 0;
    for (index, line) in markdown.lines().enumerate() {
      if !in_example && (line.starts_with('#') || line.starts_with("**输出")) {
        example_section = line == "### 示例";
      }
      if example_section && line == "```lua" && !in_example {
        in_example = true;
        source.clear();
        start = index + 1;
      } else if in_example && line == "```" {
        in_example = false;
        count += 1;
        let result = run_snippet(&entry, &source);
        if let Err(error) = result {
          failures.push(format!("{library}.md:{start}: {error}"));
        }
      } else if in_example {
        source.push_str(line);
        source.push('\n');
      }
    }
    assert!(!in_example, "unclosed example in {library}.md");
  }
  assert!(count > 150, "unexpectedly few examples: {count}");
  assert!(
    failures.is_empty(),
    "{} of {count} examples failed:\n{}",
    failures.len(),
    failures.join("\n")
  );
  println!("Executed {count} API examples with fixture assets");
}

fn run_snippet(
  entry: &std::path::Path,
  source: &str,
) -> Result<(), tg_service_lua::LuaSessionError> {
  // Run each excerpt independently and let its own callback definitions replace the fixture
  // callbacks.

  let script = format!(
    "function Init(ctx) end\nfunction HandleEvent(event) end\nfunction Update(dt) end\nfunction UpdateFrame(dt, alpha) end\nfunction Render() end\n{source}"
  );
  fs::write(entry, script).unwrap();
  let mut session = LuaService::new().create_session_with_api(
    LuaSessionSpec {
      package_id: "documentation_examples".into(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry.to_path_buf(),
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 120,
        height: 40,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: source.contains("function SaveGame("),
      save_best_enabled: source.contains("function SaveBest("),
    },
    LuaApiConfig {
      debug_enabled: true,
      ..Default::default()
    },
  )?;
  session.update()?;
  session.update_frame(Duration::from_secs_f64(1.0 / 60.0), 0.5)?;
  session.render()?;
  if source.contains("function SaveGame(") {
    session.save_game()?;
  }
  if source.contains("function SaveBest(") {
    session.save_best()?;
  }
  Ok(())
}

#[test]
fn documented_members_match_registered_libraries() {
  let root = std::env::temp_dir().join(format!("tg-doc-members-{}", std::process::id()));
  fs::create_dir(&root).unwrap();
  let fixture = Fixture(root);
  fs::create_dir(fixture.0.join("scripts")).unwrap();
  let entry = fixture.0.join("scripts/main.lua");
  let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dev_docs/zh_cn/api");
  let libraries = [
    "align",
    "base",
    "char",
    "color",
    "date",
    "timer",
    "debug",
    "draw",
    "encoding",
    "events",
    "file",
    "game",
    "i18n",
    "image",
    "ime",
    "keyboard",
    "loader",
    "math",
    "measurement",
    "random",
    "serialization",
    "slice",
    "string",
    "table",
    "utf8",
  ];
  for library in libraries {
    assert_documented_members(library, &docs, &entry);
  }
}

#[test]
fn session_object_libraries_match_documented_members() {
  let root = std::env::temp_dir().join(format!("tg-doc-input-members-{}", std::process::id()));
  fs::create_dir(&root).unwrap();
  let fixture = Fixture(root);
  fs::create_dir(fixture.0.join("scripts")).unwrap();
  let entry = fixture.0.join("scripts/main.lua");
  let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dev_docs/zh_cn/api");
  for library in ["keyboard", "ime", "random", "slice", "timer"] {
    assert_documented_members(library, &docs, &entry);
  }
}

fn assert_documented_members(library: &str, docs: &std::path::Path, entry: &std::path::Path) {
  let value_pattern = regex::Regex::new(r"(?s)### 等值\s+```(?:lua|text)\n(.*?)```").unwrap();
  let markdown = fs::read_to_string(docs.join(format!("{library}.md"))).unwrap();
  let members = markdown
    .lines()
    .filter_map(|line| line.strip_prefix("## `").and_then(|s| s.strip_suffix('`')));
  let fields = members
    .map(|name| format!("[\"{name}\"] = true"))
    .collect::<Vec<_>>()
    .join(",");
  let mut script = format!(
    r#"
    local expected = {{{fields}}}
    for name in pairs(expected) do
      debug.assert({library}[name] ~= nil, {{message = "documented member is absent: {library}." .. name}})
    end
    for name in pairs({library}) do
      debug.assert(expected[name], {{message = "registered member is undocumented: {library}." .. name}})
    end
  "#
  );
  script.push_str(
    r#"
    local function same(a, b)
      if type(a) ~= "table" or type(b) ~= "table" then return a == b end
      for key, value in pairs(a) do if not same(value, b[key]) then return false end end
      for key, value in pairs(b) do if not same(value, a[key]) then return false end end
      return true
    end
  "#,
  );
  // Separate entries before matching so a delimiter cannot consume the following constant.

  for block in markdown.split("\n## `").skip(1) {
    let name = block.split('`').next().unwrap();
    if let Some(value) = value_pattern.captures(block) {
      script.push_str(&format!(
        "\ndebug.assert(same({library}.{name}, ({})), {{message = \"constant value differs: {library}.{name}\"}})\n",
        &value[1]
      ));
    }
  }
  run_snippet(&entry, &script).unwrap_or_else(|error| panic!("{library}: {error}"));
}
