use anyhow::anyhow;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("SystemError: {cause}")]
    SystemError { cause: anyhow::Error },
}

impl From<dragonfly::Error> for Error {
    fn from(cause: dragonfly::Error) -> Self {
        Self::SystemError {
            cause: cause.into(),
        }
    }
}

impl From<dragonfly::RedisR2D2Error> for Error {
    fn from(cause: dragonfly::RedisR2D2Error) -> Self {
        Self::SystemError {
            cause: anyhow!(cause),
        }
    }
}
