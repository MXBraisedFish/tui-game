pub mod data_dirs;
pub mod host_dirs;
pub mod repair;

type EnvironmentResult<T> = Result<T, Box<dyn std::error::Error>>;

/// 先验证宿主静态资源，再确保动态数据目录存在
pub fn prepare() -> EnvironmentResult<()> {
    host_dirs::verify()?;
    data_dirs::ensure()?;
    Ok(())
}
