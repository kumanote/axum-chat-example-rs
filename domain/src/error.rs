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
