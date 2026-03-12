use anyhow::Result;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let code = tui_game::cli::actions::run_version_cli()?;
    std::process::exit(code);
}
