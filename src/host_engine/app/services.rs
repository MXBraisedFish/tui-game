//! Assembly of application service instances from a deployment root.

use super::{AsyncRuntime, EngineEventQueue};
use crate::host_engine::services::*;
use std::cell::RefCell;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Application service instances assembled under one deployment root.
///
/// # Fields
///
/// * `async_runtime` - The async runtime instance used by this owner.
/// * `engine_events` - The engine events.
/// * `audio` - The audio service instance used by this owner.
/// * `network` - The network service instance used by this owner.
/// * `random` - The random service instance used by this owner.
/// * `animation` - The animation service instance used by this owner.
/// * `screenshot` - The screenshot service instance used by this owner.
/// * `recording` - The recording service instance used by this owner.
/// * `ffmpeg` - The FFmpeg service instance used by this owner.
/// * `video` - The video service instance used by this owner.
/// * `package` - The package service instance used by this owner.
/// * `popup` - The popup service instance used by this owner.
/// * `clipboard` - The clipboard service instance used by this owner.
/// * `runtime_objects` - The runtime objects.
/// * `time` - The time service instance used by this owner.
/// * `host_objects` - The host objects.
/// * `hit_area` - The hit area service instance used by this owner.
/// * `scroll_box` - The scroll box service instance used by this owner.
/// * `progress_bar` - The progress bar service instance used by this owner.
/// * `table` - The table service instance used by this owner.
/// * `input` - The input service instance used by this owner.
/// * `input_method` - The input method service instance used by this owner.
/// * `game` - The game service instance used by this owner.
/// * `image` - The image service instance used by this owner.
/// * `screensaver` - The screensaver service instance used by this owner.
/// * `storage` - The storage service instance used by this owner.
/// * `export` - The export service instance used by this owner.
/// * `lua` - The Lua service instance used by this owner.
/// * `render` - The render service instance used by this owner.
/// * `terminal` - The terminal service instance used by this owner.
/// * `text_input` - The text input service instance used by this owner.
/// * `log` - The log service instance used by this owner.
/// * `i18n` - The i18n service instance used by this owner.
/// * `rich_text` - The rich text service instance used by this owner.
/// * `canvas` - The canvas service instance used by this owner.
/// * `layout` - The layout service instance used by this owner.
/// * `compositor` - The compositor.
/// * `presenter` - The presenter.
pub struct EngineServices {
  /// The async runtime instance used by this owner.
  pub async_runtime: AsyncRuntime,
  /// The engine events.
  pub engine_events: EngineEventQueue,
  /// The audio service instance used by this owner.
  pub audio: AudioService,
  /// The network service instance used by this owner.
  pub network: NetworkService,
  /// The random service instance used by this owner.
  pub random: RandomService,
  /// The animation service instance used by this owner.
  pub animation: AnimationService,
  /// The screenshot service instance used by this owner.
  pub screenshot: ScreenshotService,
  /// The recording service instance used by this owner.
  pub recording: RecordingService,
  /// The FFmpeg service instance used by this owner.
  pub ffmpeg: FfmpegService,
  /// The video service instance used by this owner.
  pub video: VideoService,
  /// The package service instance used by this owner.
  pub package: PackageService,
  /// The popup service instance used by this owner.
  pub popup: PopupService,
  /// The clipboard service instance used by this owner.
  pub clipboard: Rc<RefCell<ClipboardService>>,
  /// The runtime objects.
  pub runtime_objects: RuntimeObjectPool,
  /// The time service instance used by this owner.
  pub time: TimeService,
  /// The host objects.
  pub host_objects: HostObjectPool,
  /// The hit area service instance used by this owner.
  pub hit_area: HitAreaService,
  /// The scroll box service instance used by this owner.
  pub scroll_box: ScrollBoxService,
  /// The progress bar service instance used by this owner.
  pub progress_bar: ProgressBarService,
  /// The table service instance used by this owner.
  pub table: TableService,
  /// The input service instance used by this owner.
  pub input: InputService,
  /// The input method service instance used by this owner.
  pub input_method: Rc<RefCell<InputMethodService>>,
  /// The game service instance used by this owner.
  pub game: GameService,
  /// The image service instance used by this owner.
  pub image: ImageService,
  /// The screensaver service instance used by this owner.
  pub screensaver: ScreensaverService,
  /// The storage service instance used by this owner.
  pub storage: StorageService,
  /// The export service instance used by this owner.
  pub export: ExportService,
  /// The Lua service instance used by this owner.
  pub lua: LuaService,
  /// The render service instance used by this owner.
  pub render: RenderService,
  /// The terminal service instance used by this owner.
  pub terminal: TerminalService,
  /// The text input service instance used by this owner.
  pub text_input: TextInputService,
  /// The log service instance used by this owner.
  pub log: LogService,
  /// The i18n service instance used by this owner.
  pub i18n: I18nService,
  /// The rich text service instance used by this owner.
  pub rich_text: RichTextService,
  /// The canvas service instance used by this owner.
  pub canvas: CanvasService,
  /// The layout service instance used by this owner.
  pub layout: LayoutService,
  /// The compositor.
  pub compositor: FrameCompositor,
  /// The presenter.
  pub presenter: FramePresenter,
}

impl EngineServices {
  /// Create an engine services initialized from `deployment_root`.
  ///
  /// # Errors
  ///
  /// Propagate essential service initialization or deployment storage errors.
  pub fn new(deployment_root: PathBuf) -> io::Result<Self> {
    let mut log = LogService::new();
    let storage = StorageService::new(deployment_root, &mut log)?;
    Self::assemble(storage, log)
  }

  /// Construct application services against an explicitly supplied test deployment root.
  ///
  /// # Panics
  ///
  /// Panic if an internal invariant is violated: `test deployment root must accept the package
  /// log path`.
  #[cfg(test)]
  pub(super) fn for_test(root_dir: std::path::PathBuf) -> Self {
    let log = LogService::new();
    let storage = StorageService::from_root_for_test(root_dir);
    Self::assemble(storage, log).expect("test deployment root must accept the package log path")
  }

  fn assemble(storage: StorageService, mut log: LogService) -> io::Result<Self> {
    log
      .set_output_path(storage.tui_log_path())
      .map_err(|error| io::Error::new(error.kind(), format!("configure host log path: {error}")))?;
    log
      .set_package_scan_output_path(storage.package_log_path())
      .map_err(|error| {
        io::Error::new(error.kind(), format!("configure package log path: {error}"))
      })?;
    let image_cache_dir = storage.path("data/cache/images");
    let ffmpeg = FfmpegService::new(
      storage.root_dir().to_path_buf(),
      storage.cache_dir_path().join("ffmpeg"),
    );
    let async_runtime = AsyncRuntime::new();
    let audio = AudioService::new(tg_service_async::EventSink::new(
      async_runtime.event_sender(),
    ));

    Ok(Self {
      async_runtime,
      engine_events: EngineEventQueue::new(),
      audio,
      network: NetworkService::new(),
      random: RandomService::new(),
      animation: AnimationService::new(),
      screenshot: ScreenshotService::new(),
      recording: RecordingService::new(),
      ffmpeg,
      video: VideoService::new(),
      terminal: TerminalService::new(),
      clipboard: Rc::new(RefCell::new(ClipboardService::new())),
      runtime_objects: RuntimeObjectPool::new(),
      time: TimeService::new(),
      host_objects: HostObjectPool::new(),
      hit_area: HitAreaService::new(),
      scroll_box: ScrollBoxService::new(),
      progress_bar: ProgressBarService::new(),
      table: TableService::new(),
      text_input: TextInputService::new(),
      package: PackageService::new(),
      popup: PopupService::new(),
      input: InputService::new(),
      input_method: Rc::new(RefCell::new(InputMethodService::new())),
      game: GameService::new(),
      image: ImageService::new(Some(image_cache_dir)),
      screensaver: ScreensaverService::new(),
      storage,
      export: ExportService::new(),
      lua: LuaService::new(),
      render: RenderService::new(),
      log,
      i18n: I18nService::new(),
      rich_text: RichTextService::new(),
      canvas: CanvasService::new(),
      layout: LayoutService::new(),
      compositor: FrameCompositor::new(),
      presenter: FramePresenter::new(),
    })
  }
}

/// Resolve the deployment directory from the executable's parent path.
///
/// # Errors
///
/// Return an error when the executable path has no usable parent deployment directory.
pub fn deployment_root_from_executable(executable: &Path) -> io::Result<PathBuf> {
  if !executable.is_absolute() {
    return Err(io::Error::new(
      io::ErrorKind::InvalidInput,
      format!("executable path must be absolute: {}", executable.display()),
    ));
  }
  executable
    .parent()
    .filter(|parent| !parent.as_os_str().is_empty())
    .map(Path::to_path_buf)
    .ok_or_else(|| {
      io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
          "executable path has no parent directory: {}",
          executable.display()
        ),
      )
    })
}

/// Return the current deployment root.
///
/// # Errors
///
/// Propagate executable-path lookup or deployment-root resolution errors.
pub fn current_deployment_root() -> io::Result<PathBuf> {
  let executable = std::env::current_exe().map_err(|error| {
    io::Error::new(error.kind(), format!("resolve current executable: {error}"))
  })?;
  deployment_root_from_executable(&executable)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::process::Command;

  const DEPLOYMENT_ROOT_ENV: &str = "TUI_GAME_PATH_TEST_DEPLOYMENT_ROOT";

  #[test]
  fn deployment_root_is_the_executable_parent_even_with_cwd_markers() {
    let nonce = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let base = std::env::temp_dir().join(format!("tui 部署 空格-{nonce}"));
    let deployment = base.join("实际上线根");
    let caller_directory = base.join("调用目录");
    std::fs::create_dir_all(&deployment).unwrap();
    std::fs::create_dir_all(caller_directory.join("assets")).unwrap();
    std::fs::write(caller_directory.join("Cargo.toml"), "not the deployment").unwrap();

    let executable = deployment.join("tg.exe");
    let resolved = deployment_root_from_executable(&executable).unwrap();

    assert_eq!(resolved, deployment);
    assert_ne!(resolved, caller_directory);

    std::fs::remove_dir_all(base).unwrap();
  }

  #[test]
  fn deployment_root_rejects_a_non_absolute_executable_path() {
    let error = deployment_root_from_executable(std::path::Path::new("tg.exe")).unwrap_err();
    assert!(error.to_string().contains("absolute"));
  }

  #[test]
  fn production_services_stay_under_the_deployment_root_from_external_working_directories() {
    let nonce = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let base = std::env::temp_dir().join(format!("tui 部署测试 空格-{nonce}"));
    let deployment = base.join("上线 根");
    std::fs::create_dir_all(deployment.join("assets")).unwrap();
    let deployment_executable = deployment.join("tg-path-test.exe");
    std::fs::copy(std::env::current_exe().unwrap(), &deployment_executable).unwrap();

    let callers = [
      base.join("ordinary caller"),
      base.join("fake project cwd"),
      base.join("中文 与 空格 caller"),
    ];
    for (index, caller) in callers.iter().enumerate() {
      std::fs::create_dir_all(caller).unwrap();
      std::fs::write(caller.join("keep.txt"), "caller data").unwrap();
      if index == 1 {
        std::fs::create_dir_all(caller.join("assets")).unwrap();
        std::fs::write(caller.join("Cargo.toml"), "fake project marker").unwrap();
      }

      let output = Command::new(&deployment_executable)
        .arg("--exact")
        .arg("host_engine::app::services::tests::deployment_services_child_probe")
        .arg("--nocapture")
        .current_dir(caller)
        .env(DEPLOYMENT_ROOT_ENV, &deployment)
        .output()
        .unwrap();
      assert!(
        output.status.success(),
        "external CWD probe failed in {}:\n{}\n{}",
        caller.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
      );
      let mut expected_entries = vec!["keep.txt".to_string()];
      if index == 1 {
        expected_entries.extend(["Cargo.toml".to_string(), "assets".to_string()]);
      }
      expected_entries.sort();
      assert_eq!(directory_entries(caller), expected_entries);
      assert!(!caller.join("data").exists());
    }

    assert!(
      deployment
        .join("data/profiles/display_settings.json")
        .is_file()
    );
    assert!(deployment.join("data/log").is_dir());
    assert!(deployment.join("data/cache/images").is_dir());
    std::fs::remove_dir_all(base).unwrap();
  }

  #[test]
  fn deployment_services_child_probe() {
    let Some(expected_root) = std::env::var_os(DEPLOYMENT_ROOT_ENV) else {
      return;
    };
    let expected_root = PathBuf::from(expected_root);
    let current_root = current_deployment_root().unwrap();
    assert_eq!(current_root, expected_root);
    let caller = std::env::current_dir().unwrap();
    let before = directory_entries(&caller);

    let services = EngineServices::new(current_root.clone()).unwrap();

    assert_eq!(services.storage.root_dir(), current_root);
    assert!(services.storage.data_dir_path().is_dir());
    assert_eq!(directory_entries(&caller), before);
    assert!(!caller.join("data").exists());
  }

  fn directory_entries(directory: &Path) -> Vec<String> {
    let mut entries = std::fs::read_dir(directory)
      .unwrap()
      .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
      .collect::<Vec<_>>();
    entries.sort();
    entries
  }
}
