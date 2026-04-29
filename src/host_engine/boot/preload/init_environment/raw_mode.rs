//! Raw Mode 控制

type RawModeResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 启用 Raw Mode
pub fn enable() -> RawModeResult<()> {
    crossterm::terminal::enable_raw_mode()?;
    Ok(())
}

/// 禁用 Raw Mode
pub fn disable() {
    let _ = crossterm::terminal::disable_raw_mode();
}
