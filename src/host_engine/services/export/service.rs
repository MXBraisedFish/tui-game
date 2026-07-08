use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crossbeam_channel::Sender;

use crate::host_engine::services::version::{
  HOST_API_VERSION, HOST_VERSION, PACKAGE_MANIFEST_VERSION,
};
use crate::host_engine::services::{EngineEvent, LogService, StorageService, TaskId};

/// 导出文件格式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
  Zip,
  Tar,
  TarGz,
}

impl ExportFormat {
  pub fn extension(self) -> &'static str {
    match self {
      Self::Zip => "zip",
      Self::Tar => "tar",
      Self::TarGz => "tar.gz",
    }
  }
}

/// 导出范围
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportScope {
  Cache,
  Log,
  Mod,
  Profile,
  Data,
}

impl ExportScope {
  fn dir_path(self, storage: &StorageService) -> PathBuf {
    match self {
      Self::Cache => storage.cache_dir_path(),
      Self::Log => storage.log_dir_path(),
      Self::Mod => storage.mod_dir_path(),
      Self::Profile => storage.profiles_dir_path(),
      Self::Data => storage.data_dir_path(),
    }
  }

  fn dir_path_from_root(self, root_dir: &Path) -> PathBuf {
    match self {
      Self::Cache => root_dir.join("data/cache"),
      Self::Log => root_dir.join("data/log"),
      Self::Mod => root_dir.join("data/mod"),
      Self::Profile => root_dir.join("data/profiles"),
      Self::Data => root_dir.join("data"),
    }
  }
}

#[derive(Clone, Debug)]
pub struct ExportTask {
  pub scope: ExportScope,
  pub output_dir: PathBuf,
  pub file_stem: String,
  pub format: ExportFormat,
  pub root_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub enum ExportAsyncEvent {
  Started {
    task_id: TaskId,
    total: usize,
  },
  Progress {
    task_id: TaskId,
    packed: usize,
    total: usize,
  },
  Finished {
    task_id: TaskId,
    path: PathBuf,
  },
  Failed {
    task_id: TaskId,
    error: String,
  },
}

/// 收集 src_dir 下所有条目，relative 路径以 base 为基准（保留父目录名）
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

/// 导出服务：将指定目录打包为 ZIP / TAR / TAR.GZ，附带 manifest.json。
pub struct ExportService;

impl ExportService {
  pub fn new() -> Self {
    Self
  }

  /// 执行导出。`output_dir` 是用户指定的输出目录，`file_stem` 不含扩展名。
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
      log.warn(
        crate::host_engine::services::LogSource::Storage,
        format!("导出源目录不存在: {}", src_dir.display()),
      );
      return Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("源目录不存在: {}", src_dir.display()),
      ));
    }

    fs::create_dir_all(output_dir)?;

    let out_path = output_dir.join(format!("{}.{}", file_stem, format.extension()));

    // 以父目录为基准，保留源目录名（如 data/ → data/cache/...）
    let base = src_dir.parent().unwrap_or(&src_dir);
    let entries = collect_entries(base, &src_dir)?;

    self.export_entries(&out_path, format, &entries, |_| {})?;

    log.info(
      crate::host_engine::services::LogSource::Storage,
      format!("导出完成: {}", out_path.display()),
    );

    Ok(out_path)
  }

  pub fn submit_export(
    &self,
    async_runtime: &crate::host_engine::services::AsyncRuntime,
    task: ExportTask,
  ) -> TaskId {
    async_runtime.submit(crate::host_engine::services::EngineTask::Export(task))
  }

  fn export_entries<F>(
    &self,
    out_path: &Path,
    format: ExportFormat,
    entries: &[Entry],
    progress: F,
  ) -> io::Result<()>
  where
    F: FnMut(usize),
  {
    match format {
      ExportFormat::Zip => self.pack_zip(out_path, entries, progress),
      ExportFormat::Tar => self.pack_tar(out_path, entries, progress),
      ExportFormat::TarGz => self.pack_tar_gz(out_path, entries, progress),
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

  fn pack_zip<F>(&self, out: &Path, entries: &[Entry], mut progress: F) -> io::Result<()>
  where
    F: FnMut(usize),
  {
    let file = fs::File::create(out)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
      zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // manifest.json
    let mut manifest_bytes = Vec::new();
    self.write_manifest(&mut manifest_bytes)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&manifest_bytes)?;

    // directory contents
    for (index, entry) in entries.iter().enumerate() {
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

  fn pack_tar<F>(&self, out: &Path, entries: &[Entry], mut progress: F) -> io::Result<()>
  where
    F: FnMut(usize),
  {
    let file = fs::File::create(out)?;
    let mut tar = tar::Builder::new(file);

    // manifest.json
    let mut manifest_bytes = Vec::new();
    self.write_manifest(&mut manifest_bytes)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(manifest_bytes.len() as u64);
    header.set_mode(0o644);
    tar.append_data(&mut header, "manifest.json", &manifest_bytes[..])?;

    // directory contents
    for (index, entry) in entries.iter().enumerate() {
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

  fn pack_tar_gz<F>(&self, out: &Path, entries: &[Entry], mut progress: F) -> io::Result<()>
  where
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

pub fn run_export_task(
  task_id: TaskId,
  task: ExportTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  match run_export_task_inner(task_id, task, event_tx) {
    Ok(()) => Ok(()),
    Err(error) => {
      let _ = event_tx.send(EngineEvent::Export(ExportAsyncEvent::Failed {
        task_id,
        error: error.clone(),
      }));
      Err(error)
    }
  }
}

fn run_export_task_inner(
  task_id: TaskId,
  task: ExportTask,
  event_tx: &Sender<EngineEvent>,
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

  let _ = event_tx.send(EngineEvent::Export(ExportAsyncEvent::Started {
    task_id,
    total,
  }));

  let service = ExportService::new();
  service
    .export_entries(&out_path, task.format, &entries, |packed| {
      let _ = event_tx.send(EngineEvent::Export(ExportAsyncEvent::Progress {
        task_id,
        packed,
        total,
      }));
    })
    .map_err(|error| error.to_string())?;

  let _ = event_tx.send(EngineEvent::Export(ExportAsyncEvent::Finished {
    task_id,
    path: out_path,
  }));
  Ok(())
}
