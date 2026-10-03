//! Executable entry point reporting startup failures and selecting the process exit status.

mod host_engine;

fn main() {
  if let Err(error) = host_engine::run() {
    eprintln!("[Boot] failed to start TUI GAME: {error}");
    std::process::exit(1);
  }
}
