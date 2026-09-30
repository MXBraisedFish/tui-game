use super::{AsyncRuntime, EngineEventQueue};
use crate::host_engine::services::*;
use std::io;
use std::path::{Path, PathBuf};

/// Owns the host services assembled for one application run.
pub struct EngineServices {
  pub async_runtime: AsyncRuntime,
  pub engine_events: EngineEventQueue,
  pub audio: AudioService,
  pub network: NetworkService,
  pub random: RandomService,
  pub animation: AnimationService,
  pub screenshot: ScreenshotService,
  pub recording: RecordingService,
  pub ffmpeg: FfmpegService,
  pub video: VideoService,
  pub package: PackageService,
  pub popup: PopupService,
  pub clipboard: ClipboardService,
  pub runtime_objects: RuntimeObjectPool,
  pub time: TimeService,
  pub host_objects: HostObjectPool,
  pub hit_area: HitAreaService,
  pub scroll_box: ScrollBoxService,
  pub progress_bar: ProgressBarService,
  pub table: TableService,
  pub input: InputService,
  pub input_method: InputMethodService,
  pub game: GameService,
  pub image: ImageService,
  pub screensaver: ScreensaverService,
  pub storage: StorageService,
  pub export: ExportService,
  pub lua: LuaService,
  pub render: RenderService,
  pub terminal: TerminalService,
  pub text_input: TextInputService,
  pub log: LogService,
  pub i18n: I18nService,
  pub rich_text: RichTextService,
  pub canvas: CanvasService,
  pub layout: LayoutService,
  pub compositor: FrameCompositor,
  pub presenter: FramePresenter,
}

impl EngineServices {
  pub fn new(deployment_root: PathBuf) -> io::Result<Self> {
    let mut log = LogService::new();
    let storage = StorageService::new(deployment_root, &mut log)?;
    Self::assemble(storage, log)
  }

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
      clipboard: ClipboardService::new(),
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
      input_method: InputMethodService::new(),
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

/// Resolves the production root as the parent directory of the executable path.
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

/// Resolves the current process's production root without consulting its working directory.
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
