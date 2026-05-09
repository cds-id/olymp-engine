use olymp_core::error::AppError;

pub type Result<T> = std::result::Result<T, AppError>;
