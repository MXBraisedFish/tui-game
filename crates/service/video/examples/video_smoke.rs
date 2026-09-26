//! Minimal entry: a fresh video service has no active exports.

use tg_service_async::TaskId;
use tg_service_video::VideoService;

fn main() {
  let mut video = VideoService::new();
  assert_eq!(video.active_export_count(), 0);
  assert!(video.status(TaskId(1)).is_none());
  assert!(video.take_submission_feedback().is_none());
  println!("video ok: no active exports");
}
