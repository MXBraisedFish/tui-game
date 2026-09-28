use super::*;
use crate::path::{SafeRelativePath, SandboxPathKind, resolve_sandbox_path};

const MAX_MODULE_SOURCE_BYTES: usize = 1024 * 1024;
const MAX_MODULE_CHAIN_SOURCE_BYTES: usize = 4 * 1024 * 1024;
const MAX_MODULE_NESTING: usize = 16;

pub(super) fn loader(lua: &Lua, environment: &Table, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  let cache = lua.create_table()?;

  {
    let environment = environment.clone();
    let state = state.clone();
    let cache = cache.clone();
    source.raw_set(
      "require",
      lua.create_function(move |lua, values: MultiValue| {
        require_module(lua, &environment, &state, &cache, values)
      })?,
    )?;
  }

  {
    let environment = environment.clone();
    let state = state.clone();
    source.raw_set(
      "dofile",
      lua.create_function(move |lua, values: MultiValue| {
        execute_module(lua, &environment, &state, "loader.dofile", values)
      })?,
    )?;
  }

  {
    let environment = environment.clone();
    let state = state.clone();
    source.raw_set(
      "loadfile",
      lua.create_function(move |lua, values: MultiValue| {
        compile_module(lua, &environment, &state, values)
      })?,
    )?;
  }

  readonly::proxy(lua, source)
}

fn require_module(
  lua: &Lua,
  environment: &Table,
  state: &SharedApiState,
  cache: &Table,
  values: MultiValue,
) -> mlua::Result<MultiValue> {
  const METHOD: &str = "loader.require";
  let module = read_module(state, METHOD, values)?;
  if let Value::Table(cached) = cache.raw_get::<Value>(module.virtual_path.as_str())? {
    return unpack_results(&cached);
  }

  let result = execute_source(lua, environment, state, METHOD, &module)?;
  cache.raw_set(module.virtual_path.as_str(), pack_results(lua, &result)?)?;
  Ok(result)
}

fn execute_module(
  lua: &Lua,
  environment: &Table,
  state: &SharedApiState,
  method: &str,
  values: MultiValue,
) -> mlua::Result<MultiValue> {
  let module = read_module(state, method, values)?;
  execute_source(lua, environment, state, method, &module)
}

fn compile_module(
  lua: &Lua,
  environment: &Table,
  state: &SharedApiState,
  values: MultiValue,
) -> mlua::Result<Function> {
  const METHOD: &str = "loader.loadfile";
  let module = read_module(state, METHOD, values)?;
  let function = lua
    .load(&module.source)
    .set_name(&module.display_name)
    .set_mode(mlua::chunk::ChunkMode::Text)
    .set_environment(environment.clone())
    .into_function()?;
  let path = module.path;
  let source_bytes = module.source_bytes;
  let state = state.clone();

  lua.create_function(move |_lua, arguments: MultiValue| {
    with_loader_frame(&state, METHOD, &path, source_bytes, || {
      function.call::<MultiValue>(arguments)
    })
  })
}

fn execute_source(
  lua: &Lua,
  environment: &Table,
  state: &SharedApiState,
  method: &str,
  module: &LoadedModule,
) -> mlua::Result<MultiValue> {
  with_loader_frame(state, method, &module.path, module.source_bytes, || {
    lua
      .load(&module.source)
      .set_name(&module.display_name)
      .set_mode(mlua::chunk::ChunkMode::Text)
      .set_environment(environment.clone())
      .call::<MultiValue>(())
  })
}

fn with_loader_frame<T>(
  state: &SharedApiState,
  method: &str,
  path: &Path,
  source_bytes: usize,
  operation: impl FnOnce() -> mlua::Result<T>,
) -> mlua::Result<T> {
  {
    let mut api = state.borrow_mut();
    if api.loader_stack.len() >= MAX_MODULE_NESTING {
      return Err(args::message(
        method,
        format!("module nesting exceeds {MAX_MODULE_NESTING} levels"),
      ));
    }
    if api.loader_stack.iter().any(|active| active == path) {
      return Err(args::message(method, "cyclic module load detected"));
    }
    let accumulated = if api.loader_stack.is_empty() {
      source_bytes
    } else {
      api.loader_source_bytes.saturating_add(source_bytes)
    };
    if accumulated > MAX_MODULE_CHAIN_SOURCE_BYTES {
      return Err(args::message(method, "module chain source exceeds 4 MiB"));
    }
    api.loader_source_bytes = accumulated;
    api.loader_stack.push(path.to_path_buf());
  }

  let result = operation();

  let mut api = state.borrow_mut();
  api.loader_stack.pop();
  if api.loader_stack.is_empty() {
    api.loader_source_bytes = 0;
  }
  result
}

fn read_module(
  state: &SharedApiState,
  method: &str,
  values: MultiValue,
) -> mlua::Result<LoadedModule> {
  let table = args::named(method, values, &["path"])?;
  let value = args::required(&table, method, "path")?;
  let virtual_path = args::string(value, method, "path")?;
  if virtual_path.is_empty() || virtual_path.len() > 8192 || virtual_path.contains('\0') {
    return Err(args::message(method, "invalid module path"));
  }
  let scripts_root = state.borrow().context.scripts_root.clone();
  let (path, virtual_path) = resolve_loader_path(&scripts_root, &virtual_path, method)?;
  let bytes = fs::read(&path).map_err(|_| args::message(method, "module could not be read"))?;
  if bytes.len() > MAX_MODULE_SOURCE_BYTES {
    return Err(args::message(method, "module source exceeds 1 MiB"));
  }
  if bytes.first() == Some(&0x1b) {
    return Err(args::message(method, "Lua bytecode is not allowed"));
  }
  let source_bytes = bytes.len();
  let source = String::from_utf8(bytes)
    .map_err(|_| args::message(method, "module source must be UTF-8 text"))?;
  let display_name = format!("@scripts/{}", virtual_path.replace('\\', "/"));
  Ok(LoadedModule {
    path,
    virtual_path,
    display_name,
    source,
    source_bytes,
  })
}

fn pack_results(lua: &Lua, values: &MultiValue) -> mlua::Result<Table> {
  let packed = lua.create_table()?;
  packed.raw_set("n", values.len())?;
  for (index, value) in values.iter().enumerate() {
    if !matches!(value, Value::Nil) {
      packed.raw_set(index + 1, value.clone())?;
    }
  }
  Ok(packed)
}

fn unpack_results(packed: &Table) -> mlua::Result<MultiValue> {
  let length = packed.raw_get::<usize>("n")?;
  let mut values = Vec::with_capacity(length);
  for index in 1..=length {
    values.push(packed.raw_get::<Value>(index)?);
  }
  Ok(MultiValue::from_vec(values))
}

struct LoadedModule {
  path: PathBuf,
  virtual_path: String,
  display_name: String,
  source: String,
  source_bytes: usize,
}

fn resolve_loader_path(root: &Path, input: &str, method: &str) -> mlua::Result<(PathBuf, String)> {
  let mut relative = SafeRelativePath::parse(input)
    .map_err(|error| args::message(method, format!("unsafe module path: {error}")))?;
  if relative.extension().is_none() {
    relative.set_extension("lua");
  }
  if !relative
    .extension()
    .is_some_and(|value| value.eq_ignore_ascii_case("lua"))
  {
    return Err(args::message(method, "module path must use .lua"));
  }
  let path = resolve_sandbox_path(root, &relative, SandboxPathKind::File)
    .map_err(|error| args::message(method, format!("unsafe module path: {error}")))?;
  Ok((path, relative.virtual_path().to_string()))
}
