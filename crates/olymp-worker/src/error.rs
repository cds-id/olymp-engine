use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for WorkerError {
    fn from(e: sqlx::Error) -> Self {
        WorkerError::Database(e.to_string())
    }
}

impl From<redis::RedisError> for WorkerError {
    fn from(e: redis::RedisError) -> Self {
        WorkerError::Redis(e.to_string())
    }
}