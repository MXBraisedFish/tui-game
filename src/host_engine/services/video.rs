use std::{
  collections::HashMap,
  fmt, fs,
  fs::File,
  io::BufWriter,
  path::{Path, PathBuf},
};

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

use crate::host_engine::services::{
  AsyncRuntime, EngineEvent, EngineTask, RecordingExportQuality, RecordingProfile, ScreenshotRect,
  StorageService, TaskId, load_recording_playback, screenshot::TerminalFrameRasterizer,
};

#[derive(Clone, Debug)]
pub struct VideoExportTask {
  pub source_path: PathBuf,
  pub output_path: PathBuf,
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
  export_order: Vec<TaskId>,
  pending_submission_feedback: Option<bool>,
  last_failure: Option<(TaskId, VideoExportStatus)>,
}

impl VideoService {
  pub fn new() -> Self {
    Self {
      active_exports: HashMap::new(),
      output_paths: HashMap::new(),
      export_order: Vec::new(),
      pending_submission_feedback: None,
      last_failure: None,
    }
  }

  pub fn submit_recording_export(
    &mut self,
    async_runtime: &AsyncRuntime,
    storage: &StorageService,
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
      let task_id = async_runtime.submit(EngineTask::Video(VideoExportTask {
        source_path,
        output_path: output_path.clone(),
        fonts,
        profile,
      }));
      self
        .active_exports
        .insert(task_id, VideoExportStatus::Queued);
      self.output_paths.insert(task_id, output_path);
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
        let progress = export_progress(*completed_frames, *total_frames);
        self
          .active_exports
          .insert(*task_id, VideoExportStatus::Encoding(progress));
      }
      VideoAsyncEvent::Finalizing { task_id } => {
        self
          .active_exports
          .insert(*task_id, VideoExportStatus::Finalizing);
      }
      VideoAsyncEvent::Saved { task_id, .. } => {
        self.active_exports.remove(task_id);
        self.output_paths.remove(task_id);
        self.export_order.retain(|id| id != task_id);
      }
      VideoAsyncEvent::Failed { task_id, error, .. } => {
        self.last_failure = Some((*task_id, VideoExportStatus::Failed(error.clone())));
        self.active_exports.remove(task_id);
        self.output_paths.remove(task_id);
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

pub(crate) fn run_video_task(
  task_id: TaskId,
  task: VideoExportTask,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), String> {
  let _ = event_tx.send(EngineEvent::Video(VideoAsyncEvent::Preparing { task_id }));
  let temporary_path = temporary_path(&task.output_path, task_id);
  let result = export_recording(task_id, &task, &temporary_path, event_tx);
  match result {
    Ok(()) => {
      if let Err(error) = fs::rename(&temporary_path, &task.output_path) {
        let export_error = VideoExportError::new(VideoExportStage::Disk, error.to_string());
        cleanup_temporary_file(&temporary_path);
        send_failed(task_id, &task, export_error.clone(), event_tx);
        return Err(export_error.to_string());
      }
      let _ = event_tx.send(EngineEvent::Video(VideoAsyncEvent::Saved {
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

fn export_recording(
  task_id: TaskId,
  task: &VideoExportTask,
  temporary_path: &Path,
  event_tx: &Sender<EngineEvent>,
) -> Result<(), VideoExportError> {
  let playback = load_recording_playback(&task.source_path)
    .ok_or_else(|| VideoExportError::new(VideoExportStage::Parse, "recording JSON is invalid"))?;
  let metadata = playback.metadata();
  let frame_rate = task
    .profile
    .export_frame_rate
    .resolve(metadata.frame_rate, task.profile.legacy_frame_rate);
  let total_frames = sampled_frame_count(metadata.duration_us, frame_rate);
  let rasterizer = TerminalFrameRasterizer::load(&task.fonts)
    .map_err(|error| VideoExportError::new(VideoExportStage::Font, error))?;
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
      let _ = event_tx.send(EngineEvent::Video(VideoAsyncEvent::Progress {
        task_id,
        completed_frames,
        total_frames,
      }));
    }
  }

  let _ = event_tx.send(EngineEvent::Video(VideoAsyncEvent::Finalizing { task_id }));
  writer
    .as_mut()
    .expect("at least one frame is always encoded")
    .write_end()
    .map_err(|error| VideoExportError::new(VideoExportStage::Mux, error.to_string()))?;
  Ok(())
}

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
    .rate_control_mode(RateControlMode::Quality)
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
  output_path.with_extension(format!("mp4.task-{}.tmp", task_id.0))
}

fn cleanup_temporary_file(path: &Path) {
  if path.exists() {
    let _ = fs::remove_file(path);
  }
}

fn send_failed(
  task_id: TaskId,
  task: &VideoExportTask,
  error: VideoExportError,
  event_tx: &Sender<EngineEvent>,
) {
  let _ = event_tx.send(EngineEvent::Video(VideoAsyncEvent::Failed {
    task_id,
    source_path: task.source_path.clone(),
    output_path: task.output_path.clone(),
    stage: error.stage,
    error: error.message,
  }));
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::host_engine::services::RecordingPixelScale;
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
      "schema_version": 2,
      "started_at": "2026-07-21T20:20:32.895Z",
      "finished_at": "2026-07-21T20:20:32.929Z",
      "frame_rate": 30,
      "canvas": { "max_width": 2, "max_height": 1 },
      "duration_us": { "active": 33_334, "paused": 0, "wall": 33_334 },
      "palette": [
        { "text": "x", "foreground": { "type": "rgb", "value": [95, 215, 105] } },
        { "text": "y", "foreground": { "type": "rgb", "value": [238, 205, 90] } }
      ],
      "initial": { "width": 2, "height": 1, "rows": [[[2, 0]]] },
      "events": [{ "time_us": 33_333, "size": [2, 1], "changes": [[0, 1, [1]]] }]
    });
    fs::write(&source_path, serde_json::to_vec(&document).unwrap()).unwrap();
    let task = VideoExportTask {
      source_path,
      output_path: directory.join("recording.mp4"),
      fonts: Vec::new(),
      profile: RecordingProfile::default(),
    };
    let (event_tx, _event_rx) = crossbeam_channel::unbounded();

    export_recording(TaskId(1), &task, &output_path, &event_tx).unwrap();

    let size = fs::metadata(&output_path).unwrap().len();
    let mut reader =
      mp4::Mp4Reader::read_header(BufReader::new(File::open(&output_path).unwrap()), size).unwrap();
    let track = reader.tracks().get(&1).unwrap();
    assert_eq!(track.media_type().unwrap(), mp4::MediaType::H264);
    assert_eq!((track.width(), track.height()), (24, 24));
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
        assert_eq!(frame.dimensions(), (24, 24));
        decoded += 1;
      }
    }
    assert_eq!(decoded, 2);
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
      fonts: Vec::new(),
      profile: RecordingProfile::default(),
    };
    let temporary = temporary_path(&output_path, task_id);
    fs::write(&temporary, b"stale").unwrap();
    let (event_tx, event_rx) = crossbeam_channel::unbounded();

    assert!(run_video_task(task_id, task, &event_tx).is_err());
    assert!(!temporary.exists());
    assert!(event_rx.try_iter().any(|event| matches!(
      event,
      EngineEvent::Video(VideoAsyncEvent::Failed {
        task_id: TaskId(42),
        stage: VideoExportStage::Parse,
        ..
      })
    )));
    fs::remove_dir_all(directory).unwrap();
  }
}
