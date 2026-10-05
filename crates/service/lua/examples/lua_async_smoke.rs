//! Exercise asynchronous admission, string IDs, failures, and capacity reuse through public APIs.
//!
//! Run `cargo run -p tg-service-lua --example lua_async_smoke --locked`.
//! Temporary deployment assets and a single background worker require no terminal or devices.

use std::{
  fs,
  path::PathBuf,
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tg_service_async::{AsyncRuntime, TaskStatusEvent};
use tg_service_file::FileEvent;
use tg_service_image::{ImageEvent, ImageService};
use tg_service_layout::Size;
use tg_service_lua::{
  LuaEventBroker, LuaEventRoute, LuaHostCommand, LuaRoutableEvent, LuaService, LuaSessionKind,
  LuaSessionSpec, LuaSessionToken, LuaTaskOperation,
};

struct Deployment(PathBuf);

impl Drop for Deployment {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

enum Event {
  File(FileEvent),
  Image(ImageEvent),
  Status,
}

impl From<FileEvent> for Event {
  fn from(event: FileEvent) -> Self {
    Self::File(event)
  }
}

impl From<ImageEvent> for Event {
  fn from(event: ImageEvent) -> Self {
    Self::Image(event)
  }
}

impl From<TaskStatusEvent> for Event {
  fn from(_: TaskStatusEvent) -> Self {
    Self::Status
  }
}

fn main() {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let root =
    Deployment(std::env::temp_dir().join(format!("tg_lua_async_{}_{nonce}", std::process::id())));
  let assets = root.0.join("assets");
  let scripts = root.0.join("scripts");
  fs::create_dir_all(&assets).unwrap();
  fs::create_dir_all(&scripts).unwrap();
  fs::write(assets.join("input.txt"), "hello").unwrap();
  fs::write(assets.join("broken.txt"), [0xff]).unwrap();
  fs::write(assets.join("remove.txt"), "remove").unwrap();
  fs::write(assets.join("broken.png"), "invalid image").unwrap();
  image::RgbImage::new(2, 2)
    .save(assets.join("sample.png"))
    .unwrap();
  for language in ["zh_cn", "en_us"] {
    let directory = assets.join("language").join(language);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("ui.json"), r#"{"title":"ready"}"#).unwrap();
  }
  let entry = scripts.join("main.lua");
  fs::write(
    &entry,
    r#"
    local pending = {file={}, image={}, i18n={}}
    local files, images, languages = 0, 0, 0
    local failed_files, failed_images = 0, 0
    local function remember(kind, id)
      debug.assert(type(id) == "string" and not pending[kind][id])
      pending[kind][id] = true
    end
    function Init(ctx)
      debug.assert(file.read("missing.txt") == nil)
      debug.assert(image.load("missing.png") == nil)
      debug.assert(i18n.reload() == nil)
      remember("file", file.read("input.txt"))
      remember("file", file.read("input.txt", {byte=true}))
      remember("file", file.write("text.txt", "written"))
      remember("file", file.write("bytes.bin", "a\0b", {byte=true}))
      remember("file", file.create_dir("created"))
      remember("file", file.list_dir("."))
      remember("file", file.remove("remove.txt"))
      remember("file", file.read("broken.txt", {encoding="utf-8"}))
      debug.assert(file.read("input.txt") == nil)
      debug.assert(file.write("rejected.txt", "must not exist") == nil)
      for i = 1, 3 do remember("image", image.load("sample.png")) end
      remember("image", image.load("broken.png"))
      debug.assert(image.load("sample.png") == nil)
      remember("i18n", i18n.create({language_code="zh_cn", callback_language_code="en_us"}))
      debug.assert(i18n.create() == nil and i18n.reload() == nil)
    end
    function HandleEvent(e)
      debug.assert(type(e.data.request_id) == "string")
      debug.assert(pending[e.type][e.data.request_id])
      pending[e.type][e.data.request_id] = nil
      if e.type == "file" then
        files = files + 1
        if not e.data.ok then failed_files = failed_files + 1 end
        if files == 1 then
          remember("file", file.read("input.txt"))
          debug.assert(file.read("input.txt") == nil)
        end
      elseif e.type == "image" then
        images = images + 1
        if not e.data.ok then failed_images = failed_images + 1 end
      elseif e.type == "i18n" then
        debug.assert(e.data.ok and i18n.get_value("ui", "title") == "ready")
        languages = languages + 1
      end
    end
    function SaveGame()
      return {files=files, images=images, languages=languages,
        failed_files=failed_files, failed_images=failed_images}
    end
    function Update(dt) end
    function UpdateFrame(dt, alpha) end
    function Render() end
  "#,
  )
  .unwrap();
  let mut session = LuaService::new()
    .create_session(LuaSessionSpec {
      package_id: "async_smoke".into(),
      session_kind: LuaSessionKind::Game,
      entry_path: entry,
      fixed_delta: Duration::from_secs_f64(1.0 / 60.0),
      base_size: Size {
        width: 80,
        height: 24,
      },
      continue_data: None,
      best_data: None,
      save_game_enabled: true,
      save_best_enabled: false,
    })
    .unwrap();
  let runtime = AsyncRuntime::<Event>::with_worker_count(1);
  let images = ImageService::new(None);
  let mut broker = LuaEventBroker::new();
  let token = LuaSessionToken {
    kind: LuaSessionKind::Game,
    generation: 1,
  };
  broker.synchronize_sessions(Some(token), None);
  let deadline = Instant::now() + Duration::from_secs(10);
  let mut completed = 0;
  while completed < 14 && Instant::now() < deadline {
    for command in session.take_host_commands() {
      let (task_id, operation) = match command {
        LuaHostCommand::FileRequest {
          request_id,
          task,
          operation,
          virtual_path,
          event_tip,
        } => (
          runtime.submit(task),
          LuaTaskOperation::File {
            request_id,
            kind: operation,
            virtual_path,
            event_tip,
          },
        ),
        LuaHostCommand::I18nRequest {
          request_id,
          task,
          kind,
          language_code,
          callback_language_code,
        } => (
          runtime.submit(task),
          LuaTaskOperation::I18n {
            request_id,
            kind,
            language_code,
            callback_language_code,
          },
        ),
        LuaHostCommand::ImageRequest { request_id, params } => (
          images.convert_async(&runtime, params),
          LuaTaskOperation::ImageConvert { request_id },
        ),
        _ => panic!("unexpected host command"),
      };
      broker
        .register_task(task_id, token, operation, LuaEventRoute::HandleEvent)
        .unwrap();
    }
    for event in runtime.poll_events() {
      let event = match &event {
        Event::File(event) => LuaRoutableEvent::File(event),
        Event::Image(event) => LuaRoutableEvent::Image(event),
        Event::Status => continue,
      };
      broker.route_service_event(1, event).unwrap();
    }
    for delivery in broker.drain_frame(token.kind) {
      session.dispatch_event(&delivery).unwrap();
      completed += 1;
    }
    if completed < 14 {
      std::thread::sleep(Duration::from_millis(5));
    }
  }
  assert_eq!(
    completed, 14,
    "every admitted request must receive a terminal event"
  );
  let saved = session.save_game().unwrap().unwrap();
  assert_eq!(saved["files"], 9);
  assert_eq!(saved["images"], 4);
  assert_eq!(saved["languages"], 1);
  assert_eq!(saved["failed_files"], 1);
  assert_eq!(saved["failed_images"], 1);
  assert_eq!(
    fs::read_to_string(assets.join("text.txt")).unwrap(),
    "written"
  );
  assert_eq!(fs::read(assets.join("bytes.bin")).unwrap(), b"a\0b");
  assert!(assets.join("created").is_dir());
  assert!(!assets.join("remove.txt").exists());
  assert!(!assets.join("rejected.txt").exists());
  session.stop();
  drop(runtime);
  println!(
    "Lua async ok: string IDs, nil admission failures, shared quotas, terminal failures and capacity reuse"
  );
}
