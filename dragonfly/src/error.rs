use redis::RedisError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("R2D2Error: {cause:?}")]
    R2D2Error { cause: r2d2::Error },
    #[error("RedisError: {cause:?}")]
    RedisError { cause: RedisError },
}

impl From<RedisError> for Error {
    fn from(cause: RedisError) -> Self {
        Self::RedisError { cause }
    }
}

impl From<r2d2::Error> for Error {
    fn from(cause: r2d2::Error) -> Self {
        Self::R2D2Error { cause }
    }
}
