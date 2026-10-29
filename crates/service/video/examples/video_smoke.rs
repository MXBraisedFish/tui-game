//! Export a recording with the built-in encoder using an explicit deployment root.

use std::{
  fs,
  io::BufReader,
  path::PathBuf,
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tg_service_async::{AsyncRuntime, TaskState, TaskStatusEvent};
use tg_service_storage::{RecordingGpuAcceleration, RecordingProfile};
use tg_service_video::{VideoAsyncEvent, VideoExportTask};

#[derive(Debug)]
enum Event {
  Video(VideoAsyncEvent),
  Status(TaskStatusEvent),
}

impl From<VideoAsyncEvent> for Event {
  fn from(event: VideoAsyncEvent) -> Self {
    Self::Video(event)
  }
}

impl From<TaskStatusEvent> for Event {
  fn from(event: TaskStatusEvent) -> Self {
    Self::Status(event)
  }
}

struct TemporaryDirectory(PathBuf);

impl Drop for TemporaryDirectory {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn main() {
  let deployment_root = PathBuf::from(
    std::env::args_os()
      .nth(1)
      .expect("usage: video_smoke <deployment-root-with-assets>"),
  )
  .canonicalize()
  .expect("existing deployment root");
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let directory = TemporaryDirectory(std::env::temp_dir().join(format!("tg-video-smoke-{nonce}")));
  fs::create_dir(&directory.0).unwrap();
  let source_path = directory.0.join("recording.json");
  let output_path = directory.0.join("recording.mp4");
  let document = serde_json::json!({
    "schema_version": tg_core_version::MEDIA_MANIFEST_VERSION,
    "started_at": "2026-07-21T20:20:32.895Z", "finished_at": "2026-07-21T20:20:32.929Z",
    "frame_rate": 30, "canvas": { "max_width": 2, "max_height": 1 },
    "duration_us": { "active": 33_334, "paused": 0, "wall": 33_334 },
    "palette": [{ "text": "x", "foreground": { "type": "rgb", "value": [95, 215, 105] },
      "background": null, "flags": 0, "continuation": false }],
    "initial": { "width": 2, "height": 1, "rows": [[[2, 0]]] }, "events": []
  });
  fs::write(&source_path, serde_json::to_vec(&document).unwrap()).unwrap();
  let runtime = AsyncRuntime::<Event>::with_worker_count(1);
  let task_id = runtime.submit(VideoExportTask {
    source_path,
    output_path: output_path.clone(),
    ffmpeg: None,
    fonts: vec!["assets/fonts/mmo.ttf".to_string()],
    deployment_root,
    profile: RecordingProfile {
      gpu_acceleration: RecordingGpuAcceleration::Off,
      ..Default::default()
    },
  });
  let deadline = Instant::now() + Duration::from_secs(30);
  let mut saved = false;
  let mut finished = false;
  while !finished && Instant::now() < deadline {
    for event in runtime.poll_events() {
      match event {
        Event::Video(VideoAsyncEvent::Saved { mp4_path, .. }) => {
          assert_eq!(mp4_path, output_path);
          saved = true;
        }
        Event::Video(VideoAsyncEvent::Failed { error, .. }) => panic!("export failed: {error}"),
        Event::Status(TaskStatusEvent::Finished { id }) => {
          assert_eq!(id, task_id);
          finished = true;
        }
        Event::Status(status) => panic!("unexpected task status: {status:?}"),
        _ => {}
      }
    }
    std::thread::sleep(Duration::from_millis(5));
  }
  assert!(saved && finished, "export must finish within the timeout");
  assert_eq!(runtime.task_state(task_id), Some(TaskState::Finished));
  drop(runtime);
  let size = fs::metadata(&output_path).unwrap().len();
  let reader =
    mp4::Mp4Reader::read_header(BufReader::new(fs::File::open(&output_path).unwrap()), size)
      .unwrap();
  let track = reader.tracks().get(&1).unwrap();
  assert_eq!(track.media_type().unwrap(), mp4::MediaType::H264);
  assert!(track.sample_count() > 0);
  println!(
    "video ok: built-in OpenH264 exported {} frames ({size} bytes)",
    track.sample_count()
  );
}
