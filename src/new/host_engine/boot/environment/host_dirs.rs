pub fn verify() -> Result<()> {
    // assets/
    ensure_dir("assets/lang")?;
    ensure_file("assets/lang/en_us.json")?;
    ensure_dir("assets/command_lang")?;
    ensure_file("assets/command_lang/en_us.json")?;

    // scripts/
    ensure_dir("scripts/game")?;
    ensure_dir("scripts/ui")?;
    ensure_file("scripts/ui/main.lua")?;

    // textures/
    ensure_dir("textures")?;

    Ok(())
}