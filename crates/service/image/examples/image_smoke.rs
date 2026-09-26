//! Minimal entry: renders a generated PNG to half-block text, synchronously and as an async job.

use std::time::{Duration, Instant};

use tg_service_async::{AsyncRuntime, TaskStatusEvent};
use tg_service_image::{ImageConvertParams, ImageEvent, ImageService};

#[derive(Debug)]
enum Event {
  Image(ImageEvent),
  /// Executor status; this example only looks at image events.
  Status,
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
  let path = std::env::temp_dir().join(format!("tg_image_smoke_{}.png", std::process::id()));
  let mut pixels = image::RgbImage::new(8, 8);
  for (x, y, pixel) in pixels.enumerate_pixels_mut() {
    *pixel = image::Rgb([(x * 32) as u8, (y * 32) as u8, 128]);
  }
  pixels.save(&path).expect("write smoke image");

  let params = ImageConvertParams {
    image_path: path.to_string_lossy().into(),
    output_width: 8,
    output_height: 4,
    cache: false,
    ..Default::default()
  };
  let mut service = ImageService::new(None);
  let rendered = service
    .convert(params.clone())
    .expect("synchronous conversion");
  assert!(rendered.starts_with("f%"));

  let runtime = AsyncRuntime::<Event>::with_worker_count(1);
  let task = service.convert_async(&runtime, params);
  let deadline = Instant::now() + Duration::from_secs(5);
  let mut output = None;
  while output.is_none() && Instant::now() < deadline {
    for event in runtime.poll_events() {
      if let Event::Image(ImageEvent::ConvertFinished {
        task_id,
        output: text,
      }) = event
      {
        assert_eq!(task_id, task);
        output = Some(text);
      }
    }
    std::thread::sleep(Duration::from_millis(5));
  }
  assert_eq!(
    output.as_deref(),
    Some(rendered.as_str()),
    "async job renders the same text"
  );

  let _ = std::fs::remove_file(&path);
  println!("image ok: {} bytes of half-block text", rendered.len());
}
