pub fn ensure() -> Result<()> {
    // 目录
    ensure_dir("data/cache")?;
    ensure_dir("data/profiles")?;
    ensure_dir("data/profiles/mod_saves")?;
    ensure_dir("data/log")?;
    ensure_dir("data/mod")?;
    ensure_dir("data/ui")?;

    // 文件（不存在则创建空文件）
    ensure_or_create_file("data/profiles/saves.json", "{}")?;
    ensure_or_create_file("data/profiles/best_scores.json", "{}")?;
    ensure_or_create_file("data/profiles/language.txt", "en_us")?;
    ensure_or_create_file("data/log/tui_log.txt", "")?;

    Ok(())
}