use anyhow::Result;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut version_override = None;
    let mut release_url_override = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" => version_override = args.next(),
            "--release-url" => release_url_override = args.next(),
            _ => {}
        }
    }

    let code = tui_game::cli::actions::run_updata_cli(version_override, release_url_override)?;
    std::process::exit(code);
}
