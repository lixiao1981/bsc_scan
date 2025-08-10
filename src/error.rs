use thiserror::Error;

/// 统一错误类型
#[derive(Debug, Error)]
pub enum AppError {
	#[error(transparent)]
	Eyre(#[from] eyre::Report),

	#[error("Invalid argument: {0}")]
	InvalidArg(String),

	#[error("Not found: {0}")]
	NotFound(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

