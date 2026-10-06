//! Lua file library bindings with validated arguments and session-owned host access.

use super::*;
use crate::MAX_LUA_FILE_TASKS_PER_SESSION;
use crate::path::{SafeRelativePath, SandboxPathKind, sandbox_path_exists};

/// Build and register the Lua file API in the supplied VM and host context.
///
/// # Errors
///
/// Propagate Lua allocation, table construction, or function registration errors while installing
/// this library.
pub(super) fn file(lua: &Lua, state: SharedApiState) -> mlua::Result<Table> {
  let source = lua.create_table()?;
  for (name, value) in [
    ("AUTO", "auto"),
    ("ALL", "all"),
    ("CR", "cr"),
    ("LF", "lf"),
    ("CRLF", "crlf"),
    ("UTF_8", "utf-8"),
    ("UTF_16LE", "utf-16le"),
    ("UTF_16BE", "utf-16be"),
    ("GBK", "gbk"),
    ("GB18030", "gb18030"),
    ("BIG5", "big5"),
    ("SHIFT_JIS", "shift_jis"),
    ("EUC_JP", "euc-jp"),
    ("ISO_2022_JP", "iso-2022-jp"),
    ("EUC_KR", "euc-kr"),
    ("WINDOWS_874", "windows-874"),
    ("WINDOWS_1250", "windows-1250"),
    ("WINDOWS_1251", "windows-1251"),
    ("WINDOWS_1252", "windows-1252"),
    ("WINDOWS_1253", "windows-1253"),
    ("WINDOWS_1254", "windows-1254"),
    ("WINDOWS_1255", "windows-1255"),
    ("WINDOWS_1256", "windows-1256"),
    ("WINDOWS_1257", "windows-1257"),
    ("WINDOWS_1258", "windows-1258"),
    ("ISO_8859_2", "iso-8859-2"),
    ("ISO_8859_3", "iso-8859-3"),
    ("ISO_8859_4", "iso-8859-4"),
    ("ISO_8859_5", "iso-8859-5"),
    ("ISO_8859_6", "iso-8859-6"),
    ("ISO_8859_7", "iso-8859-7"),
    ("ISO_8859_8", "iso-8859-8"),
    ("ISO_8859_8_I", "iso-8859-8-i"),
    ("ISO_8859_10", "iso-8859-10"),
    ("ISO_8859_13", "iso-8859-13"),
    ("ISO_8859_14", "iso-8859-14"),
    ("ISO_8859_15", "iso-8859-15"),
    ("ISO_8859_16", "iso-8859-16"),
    ("KOI8_R", "koi8-r"),
    ("KOI8_U", "koi8-u"),
    ("IBM866", "ibm866"),
    ("MACINTOSH", "macintosh"),
    ("X_MAC_CYRILLIC", "x-mac-cyrillic"),
  ] {
    source.raw_set(name, value)?;
  }
  let read_state = state.clone();
  source.raw_set(
    "read",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "file.read";
      let parameters = args::positional(
        lua,
        method,
        values,
        &["path"],
        &["encoding", "end_of_line", "byte", "event_tip"],
      )?;
      let table = parameters.options();
      let relative_path = file_path(&parameters, method)?;
      let virtual_path = relative_path.virtual_path().to_string();
      let byte = file_byte_mode(table, method)?;
      let event_tip = file_tip(table, method)?;
      let encoding = if byte {
        None
      } else {
        validate_file_eol(table, method)?;
        Some(file_encoding(table, method)?)
      };
      let Some(path) = resolve_request_path(
        &read_state.borrow().context.assets_root,
        &relative_path,
        SandboxPathKind::File,
        method,
      )?
      else {
        return Ok(None);
      };
      let (task, operation) = match encoding {
        None => (FileTask::LuaReadBytes { path }, LuaFileOperation::ReadBytes),
        Some(encoding) => (
          FileTask::LuaReadText { path, encoding },
          LuaFileOperation::ReadText,
        ),
      };
      let request_id = enqueue_file_request(&read_state, task, operation, virtual_path, event_tip);
      Ok(request_id)
    })?,
  )?;
  let write_state = state.clone();
  source.raw_set(
    "write",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "file.write";
      require_game(&write_state.borrow(), method)?;
      let parameters = args::positional(
        lua,
        method,
        values,
        &["path", "text"],
        &["encoding", "end_of_line", "byte", "event_tip"],
      )?;
      let table = parameters.options();
      let relative_path = file_path(&parameters, method)?;
      let virtual_path = relative_path.virtual_path().to_string();
      let byte = file_byte_mode(table, method)?;
      let event_tip = file_tip(table, method)?;
      let content = parameters.required(1, method, "text")?;
      let (task, operation) = if byte {
        let bytes = file_bytes(content, method)?;
        let Some(path) = resolve_request_path(
          &write_state.borrow().context.assets_root,
          &relative_path,
          SandboxPathKind::WritableFile,
          method,
        )?
        else {
          return Ok(None);
        };
        (
          FileTask::LuaWriteBytes { path, bytes },
          LuaFileOperation::WriteBytes,
        )
      } else {
        let text = args::string(content, method, "text")?;
        if text.contains('\0') {
          return Err(args::message(method, "text must contain no NUL"));
        }
        let encoding = file_encoding(table, method)?;
        let end_of_line = validate_file_eol(table, method)?;
        let Some(path) = resolve_request_path(
          &write_state.borrow().context.assets_root,
          &relative_path,
          SandboxPathKind::WritableFile,
          method,
        )?
        else {
          return Ok(None);
        };
        (
          FileTask::LuaWriteText {
            path,
            text,
            encoding,
            end_of_line,
          },
          LuaFileOperation::WriteText,
        )
      };
      let request_id = enqueue_file_request(&write_state, task, operation, virtual_path, event_tip);
      Ok(request_id)
    })?,
  )?;
  let create_dir_state = state.clone();
  source.raw_set(
    "create_dir",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "file.create_dir";
      require_game(&create_dir_state.borrow(), method)?;
      let parameters = args::positional(lua, method, values, &["path"], &["event_tip"])?;
      let table = parameters.options();
      let relative_path = file_path(&parameters, method)?;
      let virtual_path = relative_path.virtual_path().to_string();
      let event_tip = file_tip(table, method)?;
      let assets_root = create_dir_state.borrow().context.assets_root.clone();
      let Some(path) = resolve_request_path(
        &assets_root,
        &relative_path,
        SandboxPathKind::WritableDirectory,
        method,
      )?
      else {
        return Ok(None);
      };
      let request_id = enqueue_file_request(
        &create_dir_state,
        FileTask::LuaCreateDir {
          root: assets_root,
          path,
          virtual_path: virtual_path.clone(),
        },
        LuaFileOperation::CreateDir,
        virtual_path,
        event_tip,
      );
      Ok(request_id)
    })?,
  )?;
  let exists_state = state.clone();
  source.raw_set(
    "exists",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "file.exists";
      let parameters = args::positional(lua, method, values, &["path"], &[])?;
      let value = parameters.required(0, method, "path")?;
      let path = args::string(value, method, "path")?;
      let relative_path = parse_file_path(&path, method)?;
      sandbox_path_exists(&exists_state.borrow().context.assets_root, &relative_path)
        .map_err(|error| args::message(method, format!("unsafe asset path: {error}")))
    })?,
  )?;
  let remove_state = state.clone();
  source.raw_set(
    "remove",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "file.remove";
      require_game(&remove_state.borrow(), method)?;
      let parameters =
        args::positional(lua, method, values, &["path"], &["recursive", "event_tip"])?;
      let table = parameters.options();
      let relative_path = file_path(&parameters, method)?;
      if relative_path.is_root() {
        return Err(args::message(method, "cannot remove the assets root"));
      }
      let recursive = match table.get::<Value>("recursive")? {
        Value::Nil => false,
        value => args::boolean(value, method, "recursive")?,
      };
      let virtual_path = relative_path.virtual_path().to_string();
      let event_tip = file_tip(table, method)?;
      let assets_root = remove_state.borrow().context.assets_root.clone();
      let Some(path) = resolve_request_path(
        &assets_root,
        &relative_path,
        SandboxPathKind::Removable,
        method,
      )?
      else {
        return Ok(None);
      };
      let request_id = enqueue_file_request(
        &remove_state,
        FileTask::LuaRemove {
          root: assets_root,
          path,
          virtual_path: virtual_path.clone(),
          recursive,
        },
        LuaFileOperation::Remove,
        virtual_path,
        event_tip,
      );
      Ok(request_id)
    })?,
  )?;
  let list_state = state.clone();
  source.raw_set(
    "list_dir",
    lua.create_function(move |lua, values: MultiValue| {
      let method = "file.list_dir";
      require_game(&list_state.borrow(), method)?;
      let parameters = args::positional(
        lua,
        method,
        values,
        &["path"],
        &["recursive", "file_type", "event_tip"],
      )?;
      let table = parameters.options();
      let relative_path = file_path(&parameters, method)?;
      let virtual_path = relative_path.virtual_path().to_string();
      let recursive = match table.get::<Value>("recursive")? {
        Value::Nil => false,
        value => args::boolean(value, method, "recursive")?,
      };
      let file_type = match table.get::<Value>("file_type")? {
        Value::Nil => "all".to_string(),
        value => args::string(value, method, "file_type")?,
      };
      let file_type = if file_type.eq_ignore_ascii_case("all") {
        None
      } else if file_type.is_empty()
        || file_type.starts_with('.')
        || !file_type
          .chars()
          .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '+'))
      {
        return Err(args::message(
          method,
          "file_type must be an extension such as 'rs'",
        ));
      } else {
        Some(file_type.to_ascii_lowercase())
      };
      let event_tip = file_tip(table, method)?;
      let Some(path) = resolve_request_path(
        &list_state.borrow().context.assets_root,
        &relative_path,
        SandboxPathKind::Directory,
        method,
      )?
      else {
        return Ok(None);
      };
      let request_id = enqueue_file_request(
        &list_state,
        FileTask::LuaListDir {
          path,
          recursive,
          file_type,
        },
        LuaFileOperation::ListDir,
        virtual_path,
        event_tip,
      );
      Ok(request_id)
    })?,
  )?;
  readonly::proxy(lua, source)
}

/// Validate a script-supplied relative path within the session's file sandbox.
///
/// # Errors
///
/// Return a Lua argument error when the supplied path is invalid, escapes its safe root, or is
/// not permitted for this request.
pub(super) fn file_path(
  parameters: &args::PositionalArgs,
  method: &str,
) -> mlua::Result<SafeRelativePath> {
  let path = args::string(parameters.required(0, method, "path")?, method, "path")?;
  parse_file_path(&path, method)
}

fn parse_file_path(path: &str, method: &str) -> mlua::Result<SafeRelativePath> {
  SafeRelativePath::parse(path)
    .map_err(|error| args::message(method, format!("unsafe asset path: {error}")))
}

/// Resolve the requested text encoding to one of the supported encoding names.
///
/// # Errors
///
/// Return a Lua argument error when the encoding is not one of the supported text encodings.
pub(super) fn file_encoding(table: &Table, method: &str) -> mlua::Result<String> {
  let encoding = match table.get::<Value>("encoding")? {
    Value::Nil => "auto".to_string(),
    value => args::string(value, method, "encoding")?.to_ascii_lowercase(),
  };
  let valid = encoding == "auto"
    || encoding == "utf-16le"
    || encoding == "utf-16be"
    || encoding_rs::Encoding::for_label(encoding.as_bytes()).is_some();
  if !valid {
    return Err(args::message(method, "unsupported text encoding"));
  }
  Ok(encoding)
}

fn validate_file_eol(table: &Table, method: &str) -> mlua::Result<String> {
  let value = match table.get::<Value>("end_of_line")? {
    Value::Nil => "auto".to_string(),
    value => args::string(value, method, "end_of_line")?.to_ascii_lowercase(),
  };
  if !matches!(value.as_str(), "auto" | "cr" | "lf" | "crlf") {
    return Err(args::message(method, "unsupported end_of_line value"));
  }
  Ok(value)
}

fn file_byte_mode(table: &Table, method: &str) -> mlua::Result<bool> {
  match table.get::<Value>("byte")? {
    Value::Nil => Ok(false),
    value => args::boolean(value, method, "byte"),
  }
}

fn file_bytes(value: Value, method: &str) -> mlua::Result<Vec<u8>> {
  let Value::String(value) = value else {
    return Err(args::invalid(method, "text", "string", &value));
  };
  if value.as_bytes().len() > args::MAX_API_STRING_BYTES {
    return Err(args::message(method, "parameter 'text' exceeds 1 MiB"));
  }
  Ok(value.as_bytes().to_vec())
}

/// Read an optional bounded event-tip string for the file completion payload.
///
/// # Errors
///
/// Return a Lua argument error when a supplied event-tip value is not valid bounded text.
pub(super) fn file_tip(table: &Table, method: &str) -> mlua::Result<Option<String>> {
  match table.get::<Value>("event_tip")? {
    Value::Nil => Ok(None),
    value => {
      let value = args::string(value, method, "event_tip")?;
      if value.len() > 4096 {
        Err(args::message(method, "event_tip exceeds 4 KiB"))
      } else {
        Ok(Some(value))
      }
    }
  }
}

/// Reserve a file request slot and return its string ID, or return `None` when admission is full.
///
/// # Arguments
///
/// * `state` - The session that owns the request and its result.
/// * `task` - The validated background file operation.
/// * `operation` - The result event's operation kind.
/// * `virtual_path` - The asset path visible to the script.
/// * `event_tip` - The optional text returned with the result event.
fn enqueue_file_request(
  state: &SharedApiState,
  task: FileTask,
  operation: LuaFileOperation,
  virtual_path: String,
  event_tip: Option<String>,
) -> Option<String> {
  let mut state = state.borrow_mut();
  if state.pending_file_request_ids.len() >= MAX_LUA_FILE_TASKS_PER_SESSION
    || state.commands.len() >= MAX_HOST_COMMANDS_PER_CALLBACK
  {
    return None;
  }
  let request_id = state.next_file_request_id;
  state.next_file_request_id = state.next_file_request_id.wrapping_add(1).max(1);
  state.pending_file_request_ids.insert(request_id);
  push_host_command(
    &mut state,
    LuaHostCommand::FileRequest {
      request_id,
      task,
      operation,
      virtual_path,
      event_tip,
    },
  );
  Some(request_id.to_string())
}
