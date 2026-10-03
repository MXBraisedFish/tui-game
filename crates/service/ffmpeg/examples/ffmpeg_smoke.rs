//! Independent FFmpeg smoke entry exercising the public API and checking its results.

use std::path::PathBuf;

use tg_service_ffmpeg::FfmpegService;

fn main() {
  let executable = std::env::current_exe().expect("current executable");
  let root = executable
    .parent()
    .expect("executable parent")
    .to_path_buf();
  let managed_directory = root.join(PathBuf::from("data/cache/ffmpeg"));
  let mut ffmpeg = FfmpegService::new(root, managed_directory);
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
