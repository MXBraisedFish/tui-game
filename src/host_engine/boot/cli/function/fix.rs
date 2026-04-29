type CommandResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn execute() -> CommandResult<()> {
    Ok(())
}
