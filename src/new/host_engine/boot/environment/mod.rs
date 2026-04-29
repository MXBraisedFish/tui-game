pub mod data_dirs;
pub mod host_dirs;

/// 先验证宿主目录，再确保数据目录存在
pub fn prepare() -> Result<()> {
    host_dirs::verify()?;
    data_dirs::ensure()?;
    Ok(())
}