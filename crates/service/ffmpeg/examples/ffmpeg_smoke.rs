//! Minimal entry: searches for ffmpeg next to the working directory and reports what it found.

use tg_service_ffmpeg::FfmpegService;

fn main() {
  let mut ffmpeg = FfmpegService::new(".", "data/cache/ffmpeg");
  let found = ffmpeg.refresh();
  assert_eq!(found, ffmpeg.installation().is_some());
  match ffmpeg.installation() {
    Some(installation) => println!(
      "ffmpeg ok: {} (libx264: {})",
      installation.executable().display(),
      installation.supports_encoder("libx264")
    ),
    None => println!("ffmpeg ok: not installed"),
  }
}
