pub mod kind;
pub mod manifest;
pub mod package_id;
pub mod package_id_registry;
pub mod package_manager;
pub mod registry;
pub mod scanner;
pub mod validator;

#[derive(Debug)]
pub enum PackageError {
    NotFound,
    InvalidManifest(String),
    ValidationFailed(Vec<validator::ValidationError>),
    UIDConflict(String, String),
    IOError(std::io::Error),
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(formatter, "package not found"),
            Self::InvalidManifest(message) => {
                write!(formatter, "invalid package manifest: {message}")
            }
            Self::ValidationFailed(errors) => write!(
                formatter,
                "package validation failed: {} error(s)",
                errors.len()
            ),
            Self::UIDConflict(left, right) => write!(
                formatter,
                "package uid conflict: {left} conflicts with {right}"
            ),
            Self::IOError(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for PackageError {}

impl From<std::io::Error> for PackageError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}
