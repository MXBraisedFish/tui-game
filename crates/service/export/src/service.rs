//! Service support for the export service.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crossbeam_channel::Sender;

use tg_core_atomic_fs::atomic_replace_with;
use tg_core_version::{HOST_API_VERSION, HOST_VERSION, PACKAGE_MANIFEST_VERSION};
use tg_service_async::TaskId;
use tg_service_log::LogService;
use tg_service_storage::StorageService;

/// The archive format used for a directory export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
  /// The zip setting for export format.
  Zip,
  /// The tar setting for export format.
  Tar,
  /// The tar gz setting for export format.
  TarGz,
}

impl ExportFormat {
  /// Return the current extension.
  pub fn extension(self) -> &'static str {
    match self {
      Self::Zip => "zip",
      Self::Tar => "tar",
      Self::TarGz => "tar.gz",
    }
  }
}

/// The deployment content included in a directory export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportScope {
  /// The cache setting for export scope.
  Cache,
  /// The log setting for export scope.
  Log,
  /// The mod setting for export scope.
  Mod,
  /// The profile setting for export scope.
  Profile,
  /// The screenshot setting for export scope.
  Screenshot,
  /// The recording setting for export scope.
  Recording,
  /// The data setting for export scope.
  Data,
}

impl ExportScope {
  fn dir_path(self, storage: &StorageService) -> PathBuf {
    match self {
      Self::Cache => storage.cache_dir_path(),
      Self::Log => storage.log_dir_path(),
      Self::Mod => storage.mod_dir_path(),
      Self::Profile => storage.profiles_dir_path(),
      Self::Screenshot => storage.screenshot_dir_path(),
      Self::Recording => storage.recording_dir_path(),
      Self::Data => storage.data_dir_path(),
    }
  }

  fn dir_path_from_root(self, root_dir: &Path) -> PathBuf {
    match self {
      Self::Cache => root_dir.join("data/cache"),
      Self::Log => root_dir.join("data/log"),
      Self::Mod => root_dir.join("data/mod"),
      Self::Profile => root_dir.join("data/profiles"),
      Self::Screenshot => root_dir.join("data/screenshot"),
      Self::Recording => root_dir.join("data/recording"),
      Self::Data => root_dir.join("data"),
    }
  }
}

/// The inputs of an asynchronous export operation.
///
/// # Fields
///
/// * `scope` - The set of content included in the operation.
/// * `output_dir` - The destination directory for the exported file.
/// * `file_stem` - The output filename without its format extension.
/// * `format` - The requested output format.
/// * `root_dir` - The deployment root containing assets, data, and scripts.
#[derive(Clone, Debug)]
pub struct ExportTask {
  /// The set of content included in the operation.
  pub scope: ExportScope,
  /// The destination directory for the exported file.
  pub output_dir: PathBuf,
  /// The output filename without its format extension.
  pub file_stem: String,
  /// The requested output format.
  pub format: ExportFormat,
  /// The deployment root containing assets, data, and scripts.
  pub root_dir: PathBuf,
}

/// A export async event payload queued for its owning consumer.
#[derive(Clone, Debug)]
pub enum ExportAsyncEvent {
  /// A started notification delivered to the owning consumer.
  Started {
    /// The identifier of the asynchronous task.
    task_id: TaskId,
    /// The total.
    total: usize,
  },
  /// A progress notification delivered to the owning consumer.
  Progress {
    /// The identifier of the asynchronous task.
    task_id: TaskId,
    /// The packed.
    packed: usize,
    /// The total.
    total: usize,
  },
  /// The operation is finished.
  Finished {
    /// The identifier of the asynchronous task.
    task_id: TaskId,
    /// The filesystem path to read, write, or resolve.
    path: PathBuf,
  },
  /// A failed notification delivered to the owning consumer.
  Failed {
    /// The identifier of the asynchronous task.
    task_id: TaskId,
    /// The error.
    error: String,
  },
}

fn collect_entries(base: &Path, src_dir: &Path) -> io::Result<Vec<Entry>> {
  let mut entries = Vec::new();
  collect_recursive(base, src_dir, &mut entries)?;
  entries.sort_by(|a, b| a.relative.cmp(&b.relative));
  Ok(entries)
}

struct Entry {
  relative: PathBuf,
  is_dir: bool,
  full_path: PathBuf,
}

fn collect_recursive(base: &Path, current: &Path, out: &mut Vec<Entry>) -> io::Result<()> {
  for entry in fs::read_dir(current)? {
    let entry = entry?;
    let path = entry.path();
    let relative = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
    if relative.as_os_str().is_empty() {
      continue;
    }
    let is_dir = entry.file_type()?.is_dir();
    out.push(Entry {
      relative,
      is_dir,
      full_path: path.clone(),
    });
    if is_dir {
      collect_recursive(base, &path, out)?;
    }
  }
  Ok(())
}

/// The public entry point for export operations.
#[derive(Default)]
pub struct ExportService;

impl ExportService {
  /// Create an export service with its initial state.
  pub fn new() -> Self {
    Self
  }

  /// Archive the selected deployment content and return the completed output path.
  ///
  /// # Arguments
  ///
  /// * `scope` - The set of content included in the operation.
  /// * `output_dir` - The destination directory for the exported file.
  /// * `file_stem` - The output filename without its format extension.
  /// * `format` - The requested output format.
  /// * `storage` - The deployment-relative storage service.
  /// * `log` - The service receiving diagnostic records.
  ///
  /// # Errors
  ///
  /// Return an error when the output name or directory is invalid, source entries cannot be read,
  /// or the archive cannot be created or written.
  pub fn export(
    &self,
    scope: ExportScope,
    output_dir: &Path,
    file_stem: &str,
    format: ExportFormat,
    storage: &StorageService,
    log: &mut LogService,
  ) -> io::Result<PathBuf> {
    let src_dir = scope.dir_path(storage);
    if !src_dir.is_dir() {
      log.warn_operation_failed(
        tg_core_log::LogSource::Storage,
        "export_archive",
        src_dir.display().to_string(),
        "source directory does not exist",
      );
      return Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("源目录不存在: {}", src_dir.display()),
      ));
    }

    fs::create_dir_all(output_dir)?;

    let out_path = output_dir.join(format!("{}.{}", file_stem, format.extension()));

    // Resolve archive entries from the parent so the source directory name stays in the archive.

    let base = src_dir.parent().unwrap_or(&src_dir);
    let entries = collect_entries(base, &src_dir)?;

    atomic_replace_with(&out_path, true, |temporary| {
      self.export_entries(temporary, format, &entries, || false, |_| {})
    })?;

    log.info_message(
      tg_core_log::LogSource::Storage,
      tg_core_log::HostLogMessage::new(
        "log_info.external.operation",
        "Host {operation} operation entered {state}.",
      )
      .param("operation", "export_archive")
      .param("state", format!("completed:{}", out_path.display())),
    );

    Ok(out_path)
  }

  /// Queue an archive export and return its asynchronous task identifier.
  pub fn submit_export<E>(
    &self,
    async_runtime: &tg_service_async::AsyncRuntime<E>,
    task: ExportTask,
  ) -> TaskId
  where
    E: From<ExportAsyncEvent> + From<tg_service_async::TaskStatusEvent> + Send + 'static,
  {
    async_runtime.submit(task)
  }

  fn export_entries<C, F>(
    &self,
    out_path: &Path,
    format: ExportFormat,
    entries: &[Entry],
    cancelled: C,
    progress: F,
  ) -> io::Result<()>
  where
    C: FnMut() -> bool,
    F: FnMut(usize),
  {
    match format {
      ExportFormat::Zip => self.pack_zip(out_path, entries, cancelled, progress),
      ExportFormat::Tar => self.pack_tar(out_path, entries, cancelled, progress),
      ExportFormat::TarGz => self.pack_tar_gz(out_path, entries, cancelled, progress),
    }
  }

  fn write_manifest<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let manifest = serde_json::json!({
      "version": HOST_VERSION,
      "manifest_version": PACKAGE_MANIFEST_VERSION,
      "api_version": HOST_API_VERSION,
    });
    writer.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;
    Ok(())
  }

  fn pack_zip<C, F>(
    &self,
    out: &Path,
    entries: &[Entry],
    mut cancelled: C,
    mut progress: F,
  ) -> io::Result<()>
  where
    C: FnMut() -> bool,
    F: FnMut(usize),
  {
    let file = fs::File::create(out)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
      zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut manifest_bytes = Vec::new();
    self.write_manifest(&mut manifest_bytes)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&manifest_bytes)?;

    for (index, entry) in entries.iter().enumerate() {
      ensure_not_cancelled(&mut cancelled)?;
      let relative_str = entry.relative.to_string_lossy().replace('\\', "/");
      if entry.is_dir {
        zip.add_directory(&relative_str, options)?;
      } else {
        zip.start_file(&relative_str, options)?;
        let mut file = fs::File::open(&entry.full_path)?;
        io::copy(&mut file, &mut zip)?;
      }
      progress(index + 1);
    }

    zip.finish()?;
    Ok(())
  }

  fn pack_tar<C, F>(
    &self,
    out: &Path,
    entries: &[Entry],
    mut cancelled: C,
    mut progress: F,
  ) -> io::Result<()>
  where
    C: FnMut() -> bool,
    F: FnMut(usize),
  {
    let file = fs::File::create(out)?;
    let mut tar = tar::Builder::new(file);

    let mut manifest_bytes = Vec::new();
    self.write_manifest(&mut manifest_bytes)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(manifest_bytes.len() as u64);
    header.set_mode(0o644);
    tar.append_data(&mut header, "manifest.json", &manifest_bytes[..])?;

    for (index, entry) in entries.iter().enumerate() {
      ensure_not_cancelled(&mut cancelled)?;
      if entry.is_dir {
        tar.append_dir(&entry.relative, &entry.full_path)?;
      } else {
        tar.append_file(&entry.relative, &mut fs::File::open(&entry.full_path)?)?;
      }
      progress(index + 1);
    }

    tar.finish()?;
    Ok(())
  }

  fn pack_tar_gz<C, F>(
    &self,
    out: &Path,
    entries: &[Entry],
    mut cancelled: C,
    mut progress: F,
  ) -> io::Result<()>
  where
    C: FnMut() -> bool,
    F: FnMut(usize),
  {
    let file = fs::File::create(out)?;
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);

    let mut manifest_bytes = Vec::new();
    self.write_manifest(&mut manifest_bytes)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(manifest_bytes.len() as u64);
    header.set_mode(0o644);
    tar.append_data(&mut header, "manifest.json", &manifest_bytes[..])?;

    for (index, entry) in entries.iter().enumerate() {
      ensure_not_cancelled(&mut cancelled)?;
      if entry.is_dir {
        tar.append_dir(&entry.relative, &entry.full_path)?;
      } else {
        tar.append_file(&entry.relative, &mut fs::File::open(&entry.full_path)?)?;
      }
      progress(index + 1);
    }

    let encoder = tar.into_inner()?;
    encoder.finish()?;
    Ok(())
  }
}

/// Execute an archive task and publish its progress and completion through the event sink.
///
/// # Arguments
///
/// * `task_id` - The identifier of the asynchronous task.
/// * `task` - The task.
/// * `event_tx` - The event tx.
/// * `cancellation` - The cancellation token for the operation.
///
/// # Errors
///
/// Return an error for cancellation, invalid export parameters, source access failures, or
/// archive output failures.
pub(crate) fn run_export_task<E: From<ExportAsyncEvent>>(
  task_id: TaskId,
  task: ExportTask,
  event_tx: &Sender<E>,
  cancellation: &tg_service_async::TaskCancellation,
) -> Result<(), String> {
  match run_export_task_inner(task_id, task, event_tx, cancellation) {
    Ok(()) => Ok(()),
    Err(error) => {
      let _ = event_tx.send(E::from(ExportAsyncEvent::Failed {
        task_id,
        error: error.clone(),
      }));
      Err(error)
    }
  }
}

fn run_export_task_inner<E: From<ExportAsyncEvent>>(
  task_id: TaskId,
  task: ExportTask,
  event_tx: &Sender<E>,
  cancellation: &tg_service_async::TaskCancellation,
) -> Result<(), String> {
  let src_dir = task.scope.dir_path_from_root(&task.root_dir);
  if !src_dir.is_dir() {
    return Err(format!("源目录不存在: {}", src_dir.display()));
  }

  fs::create_dir_all(&task.output_dir).map_err(|error| error.to_string())?;

  let out_path = task
    .output_dir
    .join(format!("{}.{}", task.file_stem, task.format.extension()));
  let base = src_dir.parent().unwrap_or(&src_dir);
  let entries = collect_entries(base, &src_dir).map_err(|error| error.to_string())?;
  let total = entries.len();

  let _ = event_tx.send(E::from(ExportAsyncEvent::Started { task_id, total }));

  let service = ExportService::new();
  atomic_replace_with(&out_path, true, |temporary| {
    service.export_entries(
      temporary,
      task.format,
      &entries,
      || cancellation.is_cancelled(),
      |packed| {
        let _ = event_tx.send(E::from(ExportAsyncEvent::Progress {
          task_id,
          packed,
          total,
        }));
      },
    )
  })
  .map_err(|error| error.to_string())?;

  let _ = event_tx.send(E::from(ExportAsyncEvent::Finished {
    task_id,
    path: out_path,
  }));
  Ok(())
}

fn ensure_not_cancelled(cancelled: &mut impl FnMut() -> bool) -> io::Result<()> {
  if cancelled() {
    Err(io::Error::new(
      io::ErrorKind::Interrupted,
      "export cancelled",
    ))
  } else {
    Ok(())
  }
}

impl<E: From<ExportAsyncEvent> + Send + 'static> tg_service_async::AsyncJob<E> for ExportTask {
  fn run(
    self: Box<Self>,
    id: tg_service_async::TaskId,
    events: &crossbeam_channel::Sender<E>,
    cancellation: &tg_service_async::TaskCancellation,
  ) -> Result<(), String> {
    run_export_task(id, *self, events, cancellation)
  }

  fn write_target(&self, _id: tg_service_async::TaskId) -> Option<(PathBuf, PathBuf)> {
    let target = self
      .output_dir
      .join(format!("{}.{}", self.file_stem, self.format.extension()));
    let temporary = tg_core_atomic_fs::temporary_path(&target);
    Some((target, temporary))
  }
}
