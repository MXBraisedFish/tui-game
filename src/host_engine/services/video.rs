use std::{
  collections::HashMap,
  fmt, fs,
  fs::File,
  io::{BufWriter, Write},
  path::{Path, PathBuf},
  process::{Command, Stdio},
};
use tg_service_async::AsyncRuntime;

use crossbeam_channel::Sender;
use mp4::{AvcConfig, MediaConfig, Mp4Config, Mp4Sample, Mp4Writer, TrackConfig, TrackType};
use openh264::{
  OpenH264API,
  encoder::{
    Complexity, Encoder, EncoderConfig, FrameRate, FrameType, IntraFramePeriod, QpRange,
    RateControlMode, UsageType, VuiConfig,
  },
  formats::{RgbSliceU8, YUVBuffer},
};

use crate::host_engine::services::async_runtime::TaskCancellation;
use crate::host_engine::services::{
  FfmpegInstallation, FfmpegService, RecordingExportQuality, RecordingGpuAcceleration,
  RecordingPlayback, RecordingProfile, ScreenshotRect, StorageService, TaskId,
  load_recording_playback, screenshot::TerminalFrameRasterizer,
};

#[derive(Clone, Debug)]
pub struct VideoExportTask {
  pub source_path: PathBuf,
  pub output_path: PathBuf,
  pub ffmpeg: Option<FfmpegInstallation>,
  pub fonts: Vec<String>,
  pub profile: RecordingProfile,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VideoExportProgress {
  pub completed_frames: u64,
  pub total_frames: u64,
  pub ratio: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VideoExportStatus {
  Queued,
  Preparing,
  Encoding(VideoExportProgress),
  Finalizing,
  Failed(String),
}

#[derive(Clone, Debug)]
pub enum VideoAsyncEvent {
  Preparing {
    task_id: TaskId,
  },
  Encoder {
    task_id: TaskId,
    encoder: String,
  },
  Progress {
    task_id: TaskId,
    completed_frames: u64,
    total_frames: u64,
  },
  Finalizing {
    task_id: TaskId,
  },
  Saved {
    task_id: TaskId,
    source_path: PathBuf,
    mp4_path: PathBuf,
  },
  Failed {
    task_id: TaskId,
    source_path: PathBuf,
    output_path: PathBuf,
    stage: VideoExportStage,
    error: String,
  },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoExportStage {
  Parse,
  Audio,
  Font,
  Rasterize,
  Encode,
  Mux,
  Disk,
}

impl fmt::Display for VideoExportStage {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(match self {
      Self::Parse => "parse",
      Self::Audio => "audio",
      Self::Font => "font",
      Self::Rasterize => "rasterize",
      Self::Encode => "encode",
      Self::Mux => "mux",
      Self::Disk => "disk",
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoExportError {
  pub stage: VideoExportStage,
  pub message: String,
}

impl VideoExportError {
  fn new(stage: VideoExportStage, message: impl Into<String>) -> Self {
    Self {
      stage,
      message: message.into(),
    }
  }
}

impl fmt::Display for VideoExportError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(formatter, "{}: {}", self.stage, self.message)
  }
}

impl std::error::Error for VideoExportError {}

pub struct VideoService {
  active_exports: HashMap<TaskId, VideoExportStatus>,
  output_paths: HashMap<TaskId, PathBuf>,
  source_paths: HashMap<TaskId, PathBuf>,
  export_order: Vec<TaskId>,
  pending_submission_feedback: Option<bool>,
  last_failure: Option<(TaskId, VideoExportStatus)>,
}

impl VideoService {
  pub fn new() -> Self {
    Self {
      active_exports: HashMap::new(),
      output_paths: HashMap::new(),
      source_paths: HashMap::new(),
      export_order: Vec::new(),
      pending_submission_feedback: None,
      last_failure: None,
    }
  }

  pub fn submit_recording_export<
    E: From<VideoAsyncEvent> + From<tg_service_async::TaskStatusEvent> + Send + 'static,
  >(
    &mut self,
    async_runtime: &AsyncRuntime<E>,
    storage: &StorageService,
    ffmpeg: &FfmpegService,
    source_path: PathBuf,
    fonts: Vec<String>,
    profile: RecordingProfile,
  ) -> Result<TaskId, VideoExportError> {
    let result = (|| {
      if !source_path.is_file() {
        return Err(VideoExportError::new(
          VideoExportStage::Parse,
          format!("recording does not exist: {}", source_path.display()),
        ));
      }
      fs::create_dir_all(storage.recording_dir_path())
        .map_err(|error| VideoExportError::new(VideoExportStage::Disk, error.to_string()))?;
      let output_path = next_available_output_path(
        &storage.recording_dir_path(),
        &source_path,
        self.output_paths.values(),
      );
      let task_id = async_runtime.submit(VideoExportTask {
        source_path: source_path.clone(),
        output_path: output_path.clone(),
        ffmpeg: ffmpeg.installation().cloned(),
        fonts,
        profile,
      });
      self
        .active_exports
        .insert(task_id, VideoExportStatus::Queued);
      self.output_paths.insert(task_id, output_path);
      self.source_paths.insert(task_id, source_path);
      self.export_order.push(task_id);
      Ok(task_id)
    })();
    self.pending_submission_feedback = Some(result.is_ok());
    result
  }

  pub(crate) fn take_submission_feedback(&mut self) -> Option<bool> {
    self.pending_submission_feedback.take()
  }

  pub fn handle_engine_event(&mut self, event: &VideoAsyncEvent) {
    match event {
      VideoAsyncEvent::Preparing { task_id } => {
        self
          .active_exports
          .insert(*task_id, VideoExportStatus::Preparing);
      }
      VideoAsyncEvent::Progress {
        task_id,
        completed_frames,
        total_frames,
      } => {
        let completed_frames = match self.active_exports.get(task_id) {
          Some(VideoExportStatus::Encoding(previous)) => {
            (*completed_frames).max(previous.completed_frames)
          }
          _ => *completed_frames,
        };
        let progress = export_progress(completed_frames, *total_frames);
        self
          .active_exports
          .insert(*task_id, VideoExportStatus::Encoding(progress));
      }
      VideoAsyncEvent::Finalizing { task_id } => {
        self
          .active_exports
          .insert(*task_id, VideoExportStatus::Finalizing);
      }
      VideoAsyncEvent::Encoder { .. } => {}
      VideoAsyncEvent::Saved { task_id, .. } => {
        self.active_exports.remove(task_id);
        self.output_paths.remove(task_id);
        self.source_paths.remove(task_id);
        self.export_order.retain(|id| id != task_id);
      }
      VideoAsyncEvent::Failed { task_id, error, .. } => {
        self.last_failure = Some((*task_id, VideoExportStatus::Failed(error.clone())));
        self.active_exports.remove(task_id);
        self.output_paths.remove(task_id);
        self.source_paths.remove(task_id);
        self.export_order.retain(|id| id != task_id);
      }
    }
  }

  pub fn status(&self, task_id: TaskId) -> Option<&VideoExportStatus> {
    self.active_exports.get(&task_id).or_else(|| {
      self
        .last_failure
        .as_ref()
        .filter(|(id, _)| *id == task_id)
        .map(|(_, status)| status)
    })
  }

  pub fn progress(&self, task_id: TaskId) -> Option<VideoExportProgress> {
    match self.status(task_id) {
      Some(VideoExportStatus::Encoding(progress)) => Some(*progress),
      Some(VideoExportStatus::Finalizing) => Some(VideoExportProgress {
        completed_frames: 0,
        total_frames: 0,
        ratio: 0.99,
      }),
      _ => None,
    }
  }

  pub fn active_export_count(&self) -> usize {
    self.active_exports.len()
  }

  pub fn active_task_ids(&self) -> Vec<TaskId> {
    self.export_order.clone()
  }

  pub fn is_source_exporting(&self, path: &Path) -> bool {
    self.source_paths.values().any(|source| source == path)
  }

  pub fn first_active_progress(&self) -> Option<VideoExportProgress> {
    self
      .export_order
      .iter()
      .find_map(|task_id| self.progress(*task_id))
  }
}

impl Default for VideoService {
  fn default() -> Self {
    Self::new()
  }
}

pub(crate) fn run_video_task<E: From<VideoAsyncEvent>>(
  task_id: TaskId,
  task: VideoExportTask,
  event_tx: &Sender<E>,
  cancellation: &TaskCancellation,
) -> Result<(), String> {
  let _ = event_tx.send(E::from(VideoAsyncEvent::Preparing { task_id }));
  let temporary_path = temporary_path(&task.output_path, task_id);
  let result = export_recording(task_id, &task, &temporary_path, event_tx, cancellation);
  match result {
    Ok(()) => {
      if cancellation.is_cancelled() {
        cleanup_temporary_file(&temporary_path);
        return Err("video export cancelled".to_string());
      }
      if let Err(error) = File::options()
        .read(true)
        .write(true)
        .open(&temporary_path)
        .and_then(|file| file.sync_all())
      {
        let export_error = VideoExportError::new(VideoExportStage::Disk, error.to_string());
        cleanup_temporary_file(&temporary_path);
        send_failed(task_id, &task, export_error.clone(), event_tx);
        return Err(export_error.to_string());
      }
      if let Err(error) = fs::rename(&temporary_path, &task.output_path) {
        let export_error = VideoExportError::new(VideoExportStage::Disk, error.to_string());
        cleanup_temporary_file(&temporary_path);
        send_failed(task_id, &task, export_error.clone(), event_tx);
        return Err(export_error.to_string());
      }
      let _ = event_tx.send(E::from(VideoAsyncEvent::Saved {
        task_id,
        source_path: task.source_path,
        mp4_path: task.output_path,
      }));
      Ok(())
    }
    Err(error) => {
      cleanup_temporary_file(&temporary_path);
      send_failed(task_id, &task, error.clone(), event_tx);
      Err(error.to_string())
    }
  }
}

fn export_recording<E: From<VideoAsyncEvent>>(
  task_id: TaskId,
  task: &VideoExportTask,
  temporary_path: &Path,
  event_tx: &Sender<E>,
  cancellation: &TaskCancellation,
) -> Result<(), VideoExportError> {
  let playback = load_recording_playback(&task.source_path)
    .ok_or_else(|| VideoExportError::new(VideoExportStage::Parse, "recording JSON is invalid"))?;
  let metadata = playback.metadata();
  let audio = metadata.audio.as_ref();
  if let Some(audio) = audio
    && !audio.path.is_file()
  {
    return Err(VideoExportError::new(
      VideoExportStage::Audio,
      "recording audio sidecar is missing",
    ));
  }
  let rasterizer = TerminalFrameRasterizer::load(&task.fonts)
    .map_err(|error| VideoExportError::new(VideoExportStage::Font, error))?;
  let (width, height) = TerminalFrameRasterizer::dimensions(
    metadata.max_width,
    metadata.max_height,
    task.profile.pixel_scale,
  );

  if let Some(ffmpeg) = task.ffmpeg.as_ref() {
    for encoder in usable_encoders(
      ffmpeg,
      ffmpeg_encoders(task.profile.gpu_acceleration, audio.is_some(), ffmpeg),
      width,
      height,
    ) {
      let _ = event_tx.send(E::from(VideoAsyncEvent::Encoder {
        task_id,
        encoder: encoder.to_string(),
      }));
      match export_recording_ffmpeg(
        task_id,
        task,
        temporary_path,
        event_tx,
        ffmpeg.executable(),
        encoder,
        &playback,
        &rasterizer,
        cancellation,
      ) {
        Ok(()) => return Ok(()),
        Err(_) => cleanup_temporary_file(temporary_path),
      }
    }
  }
  let _ = event_tx.send(E::from(VideoAsyncEvent::Encoder {
    task_id,
    encoder: "openh264".to_string(),
  }));
  export_recording_openh264(
    task_id,
    task,
    temporary_path,
    event_tx,
    &playback,
    &rasterizer,
    cancellation,
  )
}

fn export_recording_openh264<E: From<VideoAsyncEvent>>(
  task_id: TaskId,
  task: &VideoExportTask,
  temporary_path: &Path,
  event_tx: &Sender<E>,
  playback: &RecordingPlayback,
  rasterizer: &TerminalFrameRasterizer,
  cancellation: &TaskCancellation,
) -> Result<(), VideoExportError> {
  let metadata = playback.metadata();
  let frame_rate = task.profile.export_frame_rate.resolve(metadata.frame_rate);
  let total_frames = sampled_frame_count(metadata.duration_us, frame_rate);
  let (pixel_width, pixel_height) = TerminalFrameRasterizer::dimensions(
    metadata.max_width,
    metadata.max_height,
    task.profile.pixel_scale,
  );
  let width = u16::try_from(pixel_width).map_err(|_| {
    VideoExportError::new(VideoExportStage::Rasterize, "video width exceeds MP4 limit")
  })?;
  let height = u16::try_from(pixel_height).map_err(|_| {
    VideoExportError::new(
      VideoExportStage::Rasterize,
      "video height exceeds MP4 limit",
    )
  })?;

  let config = encoder_config(&task.profile, frame_rate);
  let mut encoder = Encoder::with_api_config(OpenH264API::from_source(), config)
    .map_err(|error| VideoExportError::new(VideoExportStage::Encode, error.to_string()))?;
  let mut frame = playback.initial_frame();
  let mut previous_frame = None;
  let mut previous_rgb = None;
  let mut cursor = 0;
  let mut writer = None;
  let mut last_progress_percent = u64::MAX;

  for frame_index in 0..total_frames {
    if cancellation.is_cancelled() {
      return Err(VideoExportError::new(
        VideoExportStage::Encode,
        "video export cancelled",
      ));
    }
    let time_us = frame_index
      .saturating_mul(1_000_000)
      .checked_div(u64::from(frame_rate))
      .unwrap_or(0);
    playback.apply_until(&mut frame, &mut cursor, time_us);
    if previous_frame.as_ref() != Some(&frame) {
      let image = rasterizer.render(
        &frame,
        ScreenshotRect {
          x: 0,
          y: 0,
          width: metadata.max_width,
          height: metadata.max_height,
        },
        task.profile.pixel_scale,
        |_, _| {},
      );
      let rgb = rgba_to_rgb(image.into_raw());
      previous_frame = Some(frame.clone());
      previous_rgb = Some(rgb);
    }
    let rgb = previous_rgb
      .as_ref()
      .expect("the first timeline frame is always rasterized");
    let yuv = YUVBuffer::from_rgb_source(RgbSliceU8::new(
      rgb,
      (usize::from(width), usize::from(height)),
    ));
    let bitstream = encoder
      .encode(&yuv)
      .map_err(|error| VideoExportError::new(VideoExportStage::Encode, error.to_string()))?;
    let encoded = mp4_frame(&bitstream)?;

    if writer.is_none() {
      let sequence_parameter_set = encoded
        .sequence_parameter_set
        .clone()
        .ok_or_else(|| VideoExportError::new(VideoExportStage::Encode, "first frame has no SPS"))?;
      let picture_parameter_set = encoded
        .picture_parameter_set
        .clone()
        .ok_or_else(|| VideoExportError::new(VideoExportStage::Encode, "first frame has no PPS"))?;
      writer = Some(start_mp4(
        temporary_path,
        width,
        height,
        frame_rate,
        sequence_parameter_set,
        picture_parameter_set,
      )?);
    }
    let sample = Mp4Sample {
      start_time: frame_index,
      duration: 1,
      rendering_offset: 0,
      is_sync: encoded.is_sync,
      bytes: encoded.sample.into(),
    };
    writer
      .as_mut()
      .expect("writer starts with first frame")
      .write_sample(1, &sample)
      .map_err(|error| VideoExportError::new(VideoExportStage::Mux, error.to_string()))?;

    let completed_frames = frame_index + 1;
    let percent = completed_frames.saturating_mul(100) / total_frames;
    if percent != last_progress_percent {
      last_progress_percent = percent;
      let _ = event_tx.send(E::from(VideoAsyncEvent::Progress {
        task_id,
        completed_frames,
        total_frames,
      }));
    }
  }

  let _ = event_tx.send(E::from(VideoAsyncEvent::Finalizing { task_id }));
  writer
    .as_mut()
    .expect("at least one frame is always encoded")
    .write_end()
    .map_err(|error| VideoExportError::new(VideoExportStage::Mux, error.to_string()))?;
  Ok(())
}

#[allow(clippy::too_many_arguments)]
fn export_recording_ffmpeg<E: From<VideoAsyncEvent>>(
  task_id: TaskId,
  task: &VideoExportTask,
  temporary_path: &Path,
  event_tx: &Sender<E>,
  ffmpeg: &Path,
  encoder: &'static str,
  playback: &RecordingPlayback,
  rasterizer: &TerminalFrameRasterizer,
  cancellation: &TaskCancellation,
) -> Result<(), VideoExportError> {
  let metadata = playback.metadata();
  let frame_rate = task.profile.export_frame_rate.resolve(metadata.frame_rate);
  let total_frames = sampled_frame_count(metadata.duration_us, frame_rate);
  let (width, height) = TerminalFrameRasterizer::dimensions(
    metadata.max_width,
    metadata.max_height,
    task.profile.pixel_scale,
  );
  let bitrate = ffmpeg_bitrate(width, height, frame_rate, task.profile.quality);
  let size = format!("{width}x{height}");
  let frame_rate_text = frame_rate.to_string();
  let bitrate_text = bitrate.to_string();
  let keyframe_interval =
    (u32::from(frame_rate) * u32::from(task.profile.keyframe_interval_seconds)).to_string();
  let audio = metadata.audio.as_ref();
  let duration_seconds = format!("{:.6}", metadata.duration_us as f64 / 1_000_000.0);

  let mut command = Command::new(ffmpeg);
  command.args([
    "-hide_banner",
    "-loglevel",
    "error",
    "-nostats",
    "-y",
    "-f",
    "rawvideo",
    "-pixel_format",
    "rgb24",
    "-video_size",
    &size,
    "-framerate",
    &frame_rate_text,
    "-i",
    "pipe:0",
  ]);
  if let Some(audio) = audio {
    command.arg("-i").arg(&audio.path);
  }
  command.args(["-map", "0:v:0"]);
  if audio.is_some() {
    command.args(["-map", "1:a:0"]);
  } else {
    command.arg("-an");
  }
  command.args([
    "-c:v",
    encoder,
    "-b:v",
    &bitrate_text,
    "-g",
    &keyframe_interval,
    "-pix_fmt",
    "yuv420p",
  ]);
  if audio.is_some() {
    command.args([
      "-c:a",
      "aac",
      "-b:a",
      "192k",
      "-af",
      "apad",
      "-t",
      &duration_seconds,
    ]);
  }
  command.args(["-f", "mp4"]);
  command
    .arg(temporary_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::piped());
  suppress_command_window(&mut command);
  let mut child = command
    .spawn()
    .map_err(|error| VideoExportError::new(VideoExportStage::Encode, error.to_string()))?;
  let mut stdin = child
    .stdin
    .take()
    .ok_or_else(|| VideoExportError::new(VideoExportStage::Encode, "FFmpeg stdin unavailable"))?;

  let mut frame = playback.initial_frame();
  let mut previous_frame = None;
  let mut previous_rgb = None;
  let mut cursor = 0;
  let mut last_progress_percent = u64::MAX;
  for frame_index in 0..total_frames {
    if cancellation.is_cancelled() {
      drop(stdin);
      let _ = child.kill();
      let _ = child.wait();
      return Err(VideoExportError::new(
        VideoExportStage::Encode,
        "video export cancelled",
      ));
    }
    let time_us = frame_index
      .saturating_mul(1_000_000)
      .checked_div(u64::from(frame_rate))
      .unwrap_or(0);
    playback.apply_until(&mut frame, &mut cursor, time_us);
    if previous_frame.as_ref() != Some(&frame) {
      let image = rasterizer.render(
        &frame,
        ScreenshotRect {
          x: 0,
          y: 0,
          width: metadata.max_width,
          height: metadata.max_height,
        },
        task.profile.pixel_scale,
        |_, _| {},
      );
      previous_frame = Some(frame.clone());
      previous_rgb = Some(rgba_to_rgb(image.into_raw()));
    }
    if let Err(error) = stdin.write_all(
      previous_rgb
        .as_deref()
        .expect("the first timeline frame is always rasterized"),
    ) {
      drop(stdin);
      let _ = child.kill();
      let _ = child.wait();
      return Err(VideoExportError::new(
        VideoExportStage::Encode,
        error.to_string(),
      ));
    }
    send_progress(
      task_id,
      frame_index + 1,
      total_frames,
      &mut last_progress_percent,
      event_tx,
    );
  }
  drop(stdin);
  let output = child
    .wait_with_output()
    .map_err(|error| VideoExportError::new(VideoExportStage::Encode, error.to_string()))?;
  if !output.status.success() {
    let detail = String::from_utf8_lossy(&output.stderr);
    let detail = detail.trim();
    let detail = if detail.is_empty() {
      "no FFmpeg error detail was reported"
    } else {
      detail
    };
    return Err(VideoExportError::new(
      VideoExportStage::Encode,
      format!(
        "FFmpeg {encoder} encoder failed with {}: {detail}",
        output.status
      ),
    ));
  }
  let _ = event_tx.send(E::from(VideoAsyncEvent::Finalizing { task_id }));
  Ok(())
}

fn ffmpeg_encoders(
  mode: RecordingGpuAcceleration,
  require_audio: bool,
  ffmpeg: &FfmpegInstallation,
) -> Vec<&'static str> {
  if require_audio && !ffmpeg.supports_encoder("aac") {
    return Vec::new();
  }
  let mut requested: Vec<&'static str> = match mode {
    RecordingGpuAcceleration::Off => Vec::new(),
    RecordingGpuAcceleration::Auto => auto_encoder_order().to_vec(),
    RecordingGpuAcceleration::Nvidia => vec!["h264_nvenc"],
    RecordingGpuAcceleration::Amd => vec!["h264_amf"],
    RecordingGpuAcceleration::Intel => vec!["h264_qsv"],
    RecordingGpuAcceleration::Apple => vec!["h264_videotoolbox"],
  };
  if ffmpeg.supports_encoder("libx264") && !requested.contains(&"libx264") {
    requested.push("libx264");
  }
  requested
    .into_iter()
    .filter(|encoder| ffmpeg.supports_encoder(encoder))
    .collect()
}

/// 用一帧黑屏真实启动一次 FFmpeg，过滤掉"列出来但打不开"的编码器。
///
/// 硬件编码器可能编译进 FFmpeg 但缺少驱动/设备，直接正式导出会让每次尝试
/// 白白跑完整条流水线后才失败。这里先用最小输入探测一次，只保留可用的候选。
fn usable_encoders(
  ffmpeg: &FfmpegInstallation,
  candidates: Vec<&'static str>,
  width: u32,
  height: u32,
) -> Vec<&'static str> {
  candidates
    .into_iter()
    .filter(|encoder| ffmpeg_encoder_works(ffmpeg.executable(), encoder, width, height))
    .collect()
}

fn ffmpeg_encoder_works(ffmpeg: &Path, encoder: &str, width: u32, height: u32) -> bool {
  let size = format!("{width}x{height}");
  let mut command = Command::new(ffmpeg);
  command.args([
    "-hide_banner",
    "-loglevel",
    "error",
    "-nostats",
    "-y",
    "-f",
    "rawvideo",
    "-pixel_format",
    "rgb24",
    "-video_size",
    &size,
    "-framerate",
    "1",
    "-i",
    "pipe:0",
    "-frames:v",
    "1",
    "-an",
    "-c:v",
    encoder,
    "-pix_fmt",
    "yuv420p",
    "-f",
    "null",
    "-",
  ]);
  command
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::null());
  suppress_command_window(&mut command);
  let mut child = match command.spawn() {
    Ok(child) => child,
    Err(_) => return false,
  };
  if let Some(mut stdin) = child.stdin.take() {
    let frame = vec![0u8; width.saturating_mul(height).saturating_mul(3) as usize];
    let _ = stdin.write_all(&frame);
  }
  child.wait().map(|status| status.success()).unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn auto_encoder_order() -> &'static [&'static str] {
  &[
    "h264_nvenc",
    "h264_qsv",
    "h264_amf",
    "h264_mf",
    "h264_videotoolbox",
  ]
}

#[cfg(target_os = "macos")]
fn auto_encoder_order() -> &'static [&'static str] {
  &["h264_videotoolbox", "h264_nvenc", "h264_qsv", "h264_amf"]
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn auto_encoder_order() -> &'static [&'static str] {
  &["h264_nvenc", "h264_qsv", "h264_amf", "h264_videotoolbox"]
}

fn ffmpeg_bitrate(
  width: u32,
  height: u32,
  frame_rate: u16,
  quality: RecordingExportQuality,
) -> u64 {
  let bits_per_pixel = match quality {
    RecordingExportQuality::Compact => 6u64,
    RecordingExportQuality::Balanced => 10,
    RecordingExportQuality::High => 16,
  };
  u64::from(width)
    .saturating_mul(u64::from(height))
    .saturating_mul(u64::from(frame_rate))
    .saturating_mul(bits_per_pixel)
    .checked_div(100)
    .unwrap_or(0)
    .clamp(250_000, 80_000_000)
}

fn send_progress<E: From<VideoAsyncEvent>>(
  task_id: TaskId,
  completed_frames: u64,
  total_frames: u64,
  last_progress_percent: &mut u64,
  event_tx: &Sender<E>,
) {
  let percent = completed_frames.saturating_mul(100) / total_frames;
  if percent == *last_progress_percent {
    return;
  }
  *last_progress_percent = percent;
  let _ = event_tx.send(E::from(VideoAsyncEvent::Progress {
    task_id,
    completed_frames,
    total_frames,
  }));
}

#[cfg(windows)]
fn suppress_command_window(command: &mut Command) {
  use std::os::windows::process::CommandExt;
  const CREATE_NO_WINDOW: u32 = 0x0800_0000;
  command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn suppress_command_window(_command: &mut Command) {}

struct EncodedMp4Frame {
  sequence_parameter_set: Option<Vec<u8>>,
  picture_parameter_set: Option<Vec<u8>>,
  sample: Vec<u8>,
  is_sync: bool,
}

fn mp4_frame(
  bitstream: &openh264::encoder::EncodedBitStream<'_>,
) -> Result<EncodedMp4Frame, VideoExportError> {
  let mut sequence_parameter_set = None;
  let mut picture_parameter_set = None;
  let mut sample = Vec::new();
  let mut is_sync = bitstream.frame_type() == FrameType::IDR;
  for layer_index in 0..bitstream.num_layers() {
    let layer = bitstream
      .layer(layer_index)
      .ok_or_else(|| VideoExportError::new(VideoExportStage::Encode, "invalid encoded layer"))?;
    for nal_index in 0..layer.nal_count() {
      let raw_nal = layer.nal_unit(nal_index).ok_or_else(|| {
        VideoExportError::new(VideoExportStage::Encode, "invalid encoded NAL unit")
      })?;
      let nal = strip_annex_b_prefix(raw_nal).ok_or_else(|| {
        VideoExportError::new(VideoExportStage::Encode, "NAL unit has no Annex-B prefix")
      })?;
      let nal_type = nal[0] & 0x1f;
      match nal_type {
        7 => sequence_parameter_set = Some(nal.to_vec()),
        8 => picture_parameter_set = Some(nal.to_vec()),
        _ => {
          is_sync |= nal_type == 5;
          let length = u32::try_from(nal.len()).map_err(|_| {
            VideoExportError::new(VideoExportStage::Encode, "NAL unit is too large")
          })?;
          sample.extend_from_slice(&length.to_be_bytes());
          sample.extend_from_slice(nal);
        }
      }
    }
  }
  if sample.is_empty() {
    return Err(VideoExportError::new(
      VideoExportStage::Encode,
      "encoded frame has no video NAL units",
    ));
  }
  Ok(EncodedMp4Frame {
    sequence_parameter_set,
    picture_parameter_set,
    sample,
    is_sync,
  })
}

fn start_mp4(
  path: &Path,
  width: u16,
  height: u16,
  frame_rate: u16,
  sequence_parameter_set: Vec<u8>,
  picture_parameter_set: Vec<u8>,
) -> Result<Mp4Writer<BufWriter<File>>, VideoExportError> {
  let file = File::create(path)
    .map_err(|error| VideoExportError::new(VideoExportStage::Disk, error.to_string()))?;
  let config = Mp4Config {
    major_brand: "isom".parse().expect("valid MP4 brand"),
    minor_version: 512,
    compatible_brands: vec![
      "isom".parse().expect("valid MP4 brand"),
      "iso2".parse().expect("valid MP4 brand"),
      "avc1".parse().expect("valid MP4 brand"),
      "mp41".parse().expect("valid MP4 brand"),
    ],
    timescale: u32::from(frame_rate),
  };
  let mut writer = Mp4Writer::write_start(BufWriter::new(file), &config)
    .map_err(|error| VideoExportError::new(VideoExportStage::Mux, error.to_string()))?;
  writer
    .add_track(&TrackConfig {
      track_type: TrackType::Video,
      timescale: u32::from(frame_rate),
      language: "und".to_string(),
      media_conf: MediaConfig::AvcConfig(AvcConfig {
        width,
        height,
        seq_param_set: sequence_parameter_set,
        pic_param_set: picture_parameter_set,
      }),
    })
    .map_err(|error| VideoExportError::new(VideoExportStage::Mux, error.to_string()))?;
  Ok(writer)
}

fn encoder_config(profile: &RecordingProfile, frame_rate: u16) -> EncoderConfig {
  let (complexity, qp) = match profile.quality {
    RecordingExportQuality::Compact => (Complexity::Low, QpRange::new(28, 40)),
    RecordingExportQuality::Balanced => (Complexity::Medium, QpRange::new(20, 34)),
    RecordingExportQuality::High => (Complexity::High, QpRange::new(12, 28)),
  };
  EncoderConfig::new()
    // OpenH264 2.6 declares SCREEN_CONTENT_NON_REAL_TIME but rejects it in
    // ParamValidationExt; screen-content real-time is the supported screen path.
    .usage_type(UsageType::ScreenContentRealTime)
    // RC_OFF keeps every timeline frame and lets the configured QP range control
    // visual quality. OpenH264 otherwise emits an initialization warning when
    // Quality RC is paired with frame skipping disabled.
    .rate_control_mode(RateControlMode::Off)
    .max_frame_rate(FrameRate::from_hz(f32::from(frame_rate)))
    .complexity(complexity)
    .qp(qp)
    .intra_frame_period(IntraFramePeriod::from_num_frames(
      u32::from(frame_rate) * u32::from(profile.keyframe_interval_seconds),
    ))
    .skip_frames(false)
    .adaptive_quantization(false)
    .background_detection(false)
    .num_threads(1)
    .vui(VuiConfig::srgb())
}

fn sampled_frame_count(duration_us: u64, frame_rate: u16) -> u64 {
  duration_us
    .saturating_mul(u64::from(frame_rate))
    .saturating_add(999_999)
    .checked_div(1_000_000)
    .unwrap_or(0)
    .max(1)
}

fn export_progress(completed_frames: u64, total_frames: u64) -> VideoExportProgress {
  let ratio = if total_frames == 0 {
    0.0
  } else {
    (completed_frames as f32 / total_frames as f32).clamp(0.0, 1.0) * 0.99
  };
  VideoExportProgress {
    completed_frames,
    total_frames,
    ratio,
  }
}

fn strip_annex_b_prefix(bytes: &[u8]) -> Option<&[u8]> {
  if bytes.starts_with(&[0, 0, 0, 1]) {
    bytes.get(4..).filter(|bytes| !bytes.is_empty())
  } else if bytes.starts_with(&[0, 0, 1]) {
    bytes.get(3..).filter(|bytes| !bytes.is_empty())
  } else {
    None
  }
}

fn rgba_to_rgb(rgba: Vec<u8>) -> Vec<u8> {
  let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
  for pixel in rgba.chunks_exact(4) {
    rgb.extend_from_slice(&pixel[..3]);
  }
  rgb
}

fn next_available_output_path<'a>(
  directory: &Path,
  source_path: &Path,
  reserved: impl IntoIterator<Item = &'a PathBuf>,
) -> PathBuf {
  let reserved = reserved.into_iter().collect::<Vec<_>>();
  let stem = source_path
    .file_stem()
    .and_then(|stem| stem.to_str())
    .filter(|stem| !stem.is_empty())
    .unwrap_or("recording");
  let direct = directory.join(format!("{stem}.mp4"));
  if !direct.exists() && !reserved.iter().any(|path| path.as_path() == direct) {
    return direct;
  }
  for suffix in 1u64.. {
    let candidate = directory.join(format!("{stem}_{suffix}.mp4"));
    if !candidate.exists() && !reserved.iter().any(|path| path.as_path() == candidate) {
      return candidate;
    }
  }
  unreachable!()
}

fn temporary_path(output_path: &Path, task_id: TaskId) -> PathBuf {
  output_path.with_extension(format!("mp4.task-{}.part", task_id.0))
}

fn cleanup_temporary_file(path: &Path) {
  if path.exists() {
    let _ = fs::remove_file(path);
  }
}

fn send_failed<E: From<VideoAsyncEvent>>(
  task_id: TaskId,
  task: &VideoExportTask,
  error: VideoExportError,
  event_tx: &Sender<E>,
) {
  let _ = event_tx.send(E::from(VideoAsyncEvent::Failed {
    task_id,
    source_path: task.source_path.clone(),
    output_path: task.output_path.clone(),
    stage: error.stage,
    error: error.message,
  }));
}

impl<E: From<VideoAsyncEvent> + Send + 'static> tg_service_async::AsyncJob<E> for VideoExportTask {
  fn run(
    self: Box<Self>,
    id: tg_service_async::TaskId,
    events: &crossbeam_channel::Sender<E>,
    cancellation: &tg_service_async::TaskCancellation,
  ) -> Result<(), String> {
    run_video_task(id, *self, events, cancellation)
  }

  /// Video exports write `<name>.mp4.task-<id>.part` so concurrent exports never share a file.
  fn write_target(&self, id: tg_service_async::TaskId) -> Option<(PathBuf, PathBuf)> {
    let temporary = self
      .output_path
      .with_extension(format!("mp4.task-{}.part", id.0));
    Some((self.output_path.clone(), temporary))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::{RecordingGpuAcceleration, RecordingPixelScale};
  use openh264::formats::YUVSource;
  use std::{
    io::BufReader,
    time::{Duration, SystemTime, UNIX_EPOCH},
  };

  fn test_directory(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let path = std::env::temp_dir().join(format!(
      "tui-game-video-{name}-{}-{nonce}",
      std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
  }

  #[test]
  fn sampled_frames_follow_the_timeline() {
    assert_eq!(sampled_frame_count(0, 30), 1);
    assert_eq!(sampled_frame_count(1_000_000, 30), 30);
    assert_eq!(sampled_frame_count(1_000_001, 30), 31);
  }

  #[test]
  fn progress_is_capped_before_finalization() {
    assert_eq!(export_progress(1, 2).ratio, 0.495);
    assert_eq!(export_progress(2, 2).ratio, 0.99);
    assert_eq!(export_progress(3, 2).ratio, 0.99);
  }

  #[test]
  fn hardware_bitrate_scales_with_quality() {
    let compact = ffmpeg_bitrate(1920, 1080, 60, RecordingExportQuality::Compact);
    let balanced = ffmpeg_bitrate(1920, 1080, 60, RecordingExportQuality::Balanced);
    let high = ffmpeg_bitrate(1920, 1080, 60, RecordingExportQuality::High);
    assert!(compact < balanced && balanced < high);
  }

  #[test]
  fn service_progress_is_monotonic_and_terminal_events_leave_the_active_queue() {
    let mut service = VideoService::new();
    let first = TaskId(10);
    let second = TaskId(11);
    service
      .active_exports
      .insert(first, VideoExportStatus::Queued);
    service
      .active_exports
      .insert(second, VideoExportStatus::Queued);
    service
      .source_paths
      .insert(first, PathBuf::from("source.json"));
    service.export_order.extend([first, second]);
    service.handle_engine_event(&VideoAsyncEvent::Progress {
      task_id: first,
      completed_frames: 25,
      total_frames: 100,
    });
    assert_eq!(service.active_export_count(), 2);
    assert_eq!(service.progress(first).unwrap().ratio, 0.2475);
    service.handle_engine_event(&VideoAsyncEvent::Finalizing { task_id: first });
    assert_eq!(service.progress(first).unwrap().ratio, 0.99);
    service.handle_engine_event(&VideoAsyncEvent::Saved {
      task_id: first,
      source_path: PathBuf::from("source.json"),
      mp4_path: PathBuf::from("output.mp4"),
    });
    assert_eq!(service.active_export_count(), 1);
    assert!(service.status(first).is_none());
    assert!(!service.is_source_exporting(Path::new("source.json")));
  }

  #[test]
  fn annex_b_prefix_is_removed() {
    assert_eq!(strip_annex_b_prefix(&[0, 0, 0, 1, 0x67]), Some(&[0x67][..]));
    assert_eq!(strip_annex_b_prefix(&[0, 0, 1, 0x68]), Some(&[0x68][..]));
    assert_eq!(strip_annex_b_prefix(&[0x65]), None);
  }

  #[test]
  fn output_name_does_not_overwrite_existing_file() {
    let directory = test_directory("output-name");
    fs::write(directory.join("demo.mp4"), b"old").unwrap();
    fs::write(directory.join("demo_1.mp4"), b"old").unwrap();
    assert_eq!(
      next_available_output_path(
        &directory,
        Path::new("demo.json"),
        std::iter::empty::<&PathBuf>(),
      ),
      directory.join("demo_2.mp4")
    );
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn pixel_scale_dimensions_are_even() {
    for scale in [
      RecordingPixelScale::Half,
      RecordingPixelScale::Original,
      RecordingPixelScale::Double,
    ] {
      let (width, height) = TerminalFrameRasterizer::dimensions(3, 5, scale);
      assert_eq!(width % 2, 0);
      assert_eq!(height % 2, 0);
    }
  }

  #[test]
  fn short_recording_exports_as_decodable_h264_mp4() {
    let directory = test_directory("encode");
    let source_path = directory.join("recording.json");
    let output_path = directory.join("recording.mp4.tmp");
    let document = serde_json::json!({
      "schema_version": crate::host_engine::services::MEDIA_MANIFEST_VERSION,
      "started_at": "2026-07-21T20:20:32.895Z",
      "finished_at": "2026-07-21T20:20:32.929Z",
      "frame_rate": 30,
      "canvas": { "max_width": 2, "max_height": 1 },
      "duration_us": { "active": 33_334, "paused": 0, "wall": 33_334 },
      "palette": [
        { "text": "x", "foreground": { "type": "rgb", "value": [95, 215, 105] }, "background": null, "flags": 0, "continuation": false },
        { "text": "y", "foreground": { "type": "rgb", "value": [238, 205, 90] }, "background": null, "flags": 0, "continuation": false }
      ],
      "initial": { "width": 2, "height": 1, "rows": [[[2, 0]]] },
      "events": [{ "time_us": 33_333, "size": [2, 1], "changes": [[0, 1, [1]]] }]
    });
    fs::write(&source_path, serde_json::to_vec(&document).unwrap()).unwrap();
    let task = VideoExportTask {
      source_path,
      output_path: directory.join("recording.mp4"),
      ffmpeg: None,
      fonts: Vec::new(),
      profile: RecordingProfile {
        gpu_acceleration: RecordingGpuAcceleration::Off,
        ..Default::default()
      },
    };
    let (event_tx, _event_rx) = crossbeam_channel::unbounded::<VideoAsyncEvent>();

    export_recording(
      TaskId(1),
      &task,
      &output_path,
      &event_tx,
      &TaskCancellation::new(TaskId(1)),
    )
    .unwrap();

    let size = fs::metadata(&output_path).unwrap().len();
    let mut reader =
      mp4::Mp4Reader::read_header(BufReader::new(File::open(&output_path).unwrap()), size).unwrap();
    let track = reader.tracks().get(&1).unwrap();
    assert_eq!(track.media_type().unwrap(), mp4::MediaType::H264);
    assert_eq!((track.width(), track.height()), (36, 36));
    assert_eq!(track.timescale(), 30);
    assert_eq!(track.sample_count(), 2);
    assert_eq!(track.duration(), Duration::from_micros(66_666));
    let avcc = &track.trak.mdia.minf.stbl.stsd.avc1.as_ref().unwrap().avcc;
    assert!(!avcc.sequence_parameter_sets.is_empty());
    assert!(!avcc.picture_parameter_sets.is_empty());
    let sps = avcc.sequence_parameter_sets[0].bytes.clone();
    let pps = avcc.picture_parameter_sets[0].bytes.clone();

    let mut decoder = openh264::decoder::Decoder::new().unwrap();
    let mut decoded = 0;
    for sample_id in 1..=track.sample_count() {
      let sample = reader.read_sample(1, sample_id).unwrap().unwrap();
      let mut annex_b = Vec::new();
      if sample_id == 1 {
        for parameter_set in [&sps, &pps] {
          annex_b.extend_from_slice(&[0, 0, 0, 1]);
          annex_b.extend_from_slice(parameter_set);
        }
      }
      let mut offset = 0;
      while offset < sample.bytes.len() {
        let length =
          u32::from_be_bytes(sample.bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        annex_b.extend_from_slice(&[0, 0, 0, 1]);
        annex_b.extend_from_slice(&sample.bytes[offset..offset + length]);
        offset += length;
      }
      if let Some(frame) = decoder.decode(&annex_b).unwrap() {
        assert_eq!(frame.dimensions(), (36, 36));
        decoded += 1;
      }
    }
    assert_eq!(decoded, 2);
    drop(reader);
    fs::remove_dir_all(directory).unwrap();
  }

  fn write_recording_with_audio(directory: &Path) -> PathBuf {
    let source_path = directory.join("recording.json");
    let audio_path = directory.join("recording.audio.wav");
    let mut wav = hound::WavWriter::create(
      &audio_path,
      hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
      },
    )
    .unwrap();
    for _ in 0..4_800 {
      wav.write_sample(0i16).unwrap();
      wav.write_sample(0i16).unwrap();
    }
    wav.finalize().unwrap();
    let document = serde_json::json!({
      "schema_version": crate::host_engine::services::MEDIA_MANIFEST_VERSION,
      "started_at": "2026-07-21T20:20:32.895Z",
      "finished_at": "2026-07-21T20:20:32.995Z",
      "frame_rate": 30,
      "canvas": { "max_width": 2, "max_height": 1 },
      "duration_us": { "active": 100_000, "paused": 0, "wall": 100_000 },
      "audio": {
        "file": "recording.audio.wav",
        "codec": "pcm_s16le",
        "sample_rate": 48_000,
        "channels": 2,
        "duration_us": 100_000
      },
      "palette": [
        { "text": "x", "foreground": { "type": "rgb", "value": [95, 215, 105] }, "background": null, "flags": 0, "continuation": false }
      ],
      "initial": { "width": 2, "height": 1, "rows": [[[2, 0]]] },
      "events": []
    });
    fs::write(&source_path, serde_json::to_vec(&document).unwrap()).unwrap();
    source_path
  }

  #[test]
  fn recording_audio_is_omitted_when_ffmpeg_is_unavailable() {
    let directory = test_directory("audio-fallback");
    let source_path = write_recording_with_audio(&directory);
    let output_path = directory.join("recording.mp4.tmp");
    let task = VideoExportTask {
      source_path,
      output_path: directory.join("recording.mp4"),
      ffmpeg: None,
      fonts: Vec::new(),
      profile: RecordingProfile {
        gpu_acceleration: RecordingGpuAcceleration::Auto,
        ..Default::default()
      },
    };
    let (event_tx, _event_rx) = crossbeam_channel::unbounded::<VideoAsyncEvent>();

    export_recording(
      TaskId(2),
      &task,
      &output_path,
      &event_tx,
      &TaskCancellation::new(TaskId(2)),
    )
    .unwrap();

    let size = fs::metadata(&output_path).unwrap().len();
    let reader =
      mp4::Mp4Reader::read_header(BufReader::new(File::open(&output_path).unwrap()), size).unwrap();
    assert_eq!(reader.tracks().len(), 1);
    assert_eq!(
      reader.tracks().get(&1).unwrap().media_type().unwrap(),
      mp4::MediaType::H264
    );
    drop(reader);
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  #[ignore = "requires an external FFmpeg sidecar with H.264/AAC support"]
  fn recording_with_audio_exports_as_h264_aac_mp4() {
    let directory = test_directory("audio-encode");
    let source_path = write_recording_with_audio(&directory);
    let output_path = directory.join("recording.mp4.tmp");
    let ffmpeg = FfmpegService::new(&directory, directory.join("ffmpeg"));
    let task = VideoExportTask {
      source_path,
      output_path: directory.join("recording.mp4"),
      ffmpeg: ffmpeg.installation().cloned(),
      fonts: Vec::new(),
      profile: RecordingProfile {
        gpu_acceleration: RecordingGpuAcceleration::Off,
        ..Default::default()
      },
    };
    let (event_tx, _event_rx) = crossbeam_channel::unbounded::<VideoAsyncEvent>();

    export_recording(
      TaskId(2),
      &task,
      &output_path,
      &event_tx,
      &TaskCancellation::new(TaskId(2)),
    )
    .unwrap();

    let size = fs::metadata(&output_path).unwrap().len();
    let reader =
      mp4::Mp4Reader::read_header(BufReader::new(File::open(&output_path).unwrap()), size).unwrap();
    assert_eq!(reader.tracks().len(), 2);
    assert!(
      reader
        .tracks()
        .values()
        .any(|track| track.media_type().unwrap() == mp4::MediaType::H264)
    );
    assert!(
      reader
        .tracks()
        .values()
        .any(|track| track.media_type().unwrap() == mp4::MediaType::AAC)
    );
    drop(reader);
    fs::remove_dir_all(directory).unwrap();
  }

  #[test]
  fn failed_export_removes_its_temporary_file() {
    let directory = test_directory("failure-cleanup");
    let source_path = directory.join("broken.json");
    let output_path = directory.join("broken.mp4");
    fs::write(&source_path, "{").unwrap();
    let task_id = TaskId(42);
    let task = VideoExportTask {
      source_path,
      output_path: output_path.clone(),
      ffmpeg: None,
      fonts: Vec::new(),
      profile: RecordingProfile {
        gpu_acceleration: RecordingGpuAcceleration::Off,
        ..Default::default()
      },
    };
    let temporary = temporary_path(&output_path, task_id);
    fs::write(&temporary, b"stale").unwrap();
    let (event_tx, event_rx) = crossbeam_channel::unbounded::<VideoAsyncEvent>();

    assert!(run_video_task(task_id, task, &event_tx, &TaskCancellation::new(task_id)).is_err());
    assert!(!temporary.exists());
    assert!(event_rx.try_iter().any(|event| matches!(
      event,
      VideoAsyncEvent::Failed {
        task_id: TaskId(42),
        stage: VideoExportStage::Parse,
        ..
      }
    )));
    fs::remove_dir_all(directory).unwrap();
  }
}
