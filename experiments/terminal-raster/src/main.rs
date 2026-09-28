#[cfg(windows)]
mod capture;
mod fixture;
#[cfg(windows)]
mod native;
mod software;

use crossterm::{
  cursor::{Hide, Show},
  event::{self, Event, KeyCode},
  execute,
  terminal::{
    self, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
  },
};
use fixture::{COLS, Profile, ROWS};
use std::{
  fs,
  io::{self, Write},
  path::Path,
  time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct TerminalGuard;
impl Drop for TerminalGuard {
  fn drop(&mut self) {
    let _ = write!(io::stdout(), "\x1b[0m");
    let _ = execute!(io::stdout(), EnableLineWrap, Show, LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
  }
}

fn emit(title: &str, seconds: u64, metadata: &Path) -> Result<()> {
  if title.chars().any(char::is_control) {
    return Err("title contains control characters".into());
  }
  let (cols, rows) = terminal::size()?;
  if cols < u16::try_from(COLS)? || rows < u16::try_from(ROWS)? {
    return Err(
      format!(
        "fixture needs {COLS}x{ROWS} cells, terminal has {cols}x{rows}; enlarge window and retry"
      )
      .into(),
    );
  }
  let mut out = io::stdout();
  terminal::enable_raw_mode()?;
  let _guard = TerminalGuard;
  execute!(
    out,
    EnterAlternateScreen,
    DisableLineWrap,
    Hide,
    SetTitle(title)
  )?;
  draw_fixture(&mut out, title, seconds, metadata)?;
  let start = Instant::now();
  while start.elapsed() < Duration::from_secs(seconds) {
    if event::poll(Duration::from_millis(100))? {
      match event::read()? {
        Event::Key(key)
          if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c')
          ) =>
        {
          break;
        }
        Event::Resize(..) => draw_fixture(&mut out, title, seconds, metadata)?,
        _ => {}
      }
    }
  }
  Ok(())
}

fn draw_fixture(out: &mut impl Write, title: &str, seconds: u64, metadata: &Path) -> Result<()> {
  let (cols, rows) = terminal::size()?;
  if u32::from(cols) < COLS || u32::from(rows) < ROWS {
    return Err("fixture clipped after resize; enlarge window".into());
  }
  write!(out, "\x1b[0m\x1b[2J\x1b[H")?;
  for run in fixture::runs() {
    write!(
      out,
      "\x1b[{};{}H\x1b[0;{};{}m",
      run.y + 1,
      run.x + 1,
      run.fg.sgr(false),
      run.bg.sgr(true)
    )?;
    if run.bold {
      write!(out, "\x1b[1m")?;
    }
    if run.italic {
      write!(out, "\x1b[3m")?;
    }
    if run.underline {
      write!(out, "\x1b[4m")?;
    }
    if run.reverse {
      write!(out, "\x1b[7m")?;
    }
    write!(out, "{}", run.text)?;
  }
  out.flush()?;
  fs::write(
    metadata,
    serde_json::to_vec_pretty(
      &serde_json::json!({"title":title,"cols":cols,"rows":rows,"fixture_cols":COLS,"fixture_rows":ROWS,"wait_seconds":seconds,"ready":true}),
    )?,
  )?;
  Ok(())
}

fn render(profile: &Path, out: &Path) -> Result<()> {
  let profile: Profile = serde_json::from_slice(&fs::read(profile)?)?;
  profile.validate()?;
  fs::create_dir_all(out)?;
  fs::write(
    out.join("profile.json"),
    serde_json::to_vec_pretty(&profile)?,
  )?;
  let runs = fixture::runs();
  fs::write(out.join("fixture.json"), serde_json::to_vec_pretty(&runs)?)?;
  let font = software::SoftwareFont::load(&profile)?;
  for (name, fit) in [("software-natural", false), ("software-fit", true)] {
    let (image, report) = font.render(&profile, &runs, fit)?;
    image.save(out.join(format!("{name}.png")))?;
    fs::write(
      out.join(format!("{name}.json")),
      serde_json::to_vec_pretty(&report)?,
    )?;
  }
  #[cfg(windows)]
  for (name, mode, cleartype) in [
    ("native-cell-gray", "cell", false),
    ("native-cell-cleartype", "cell", true),
    ("native-run-gray", "run", false),
    ("native-cluster-gray", "cluster", false),
  ] {
    let (image, report) = native::render(&profile, &runs, mode, cleartype)?;
    image.save(out.join(format!("{name}.png")))?;
    fs::write(
      out.join(format!("{name}.json")),
      serde_json::to_vec_pretty(&report)?,
    )?;
  }
  println!("Generated controlled probes in {}", out.display());
  Ok(())
}

fn main() -> Result<()> {
  std::panic::set_hook(Box::new(|info| {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
    eprintln!("{info}");
  }));
  let args: Vec<_> = std::env::args().skip(1).collect();
  match args.as_slice() {
        [cmd,profile,out] if cmd=="render"=>render(Path::new(profile),Path::new(out)),
        [cmd,title,seconds,metadata] if cmd=="emit"=>emit(title,seconds.parse()?,Path::new(metadata)),
        #[cfg(windows)]
        [cmd,title,png] if cmd=="capture"=>capture::capture(title,Path::new(png)),
        #[cfg(windows)]
        [cmd,title] if cmd=="close"=>capture::close(title),
        _=>Err("usage: terminal-raster render <profile.json> <output-dir> | emit <unique-title> <seconds> <ready.json> | capture <unique-title> <window.png>".into())
    }
}
