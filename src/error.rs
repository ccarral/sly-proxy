use std::fmt;
use std::io;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum SlyError {
    IoError(std::io::Error),
    JoinError(JoinError),
    SendError(String),
    Generic(String),
}

impl fmt::Display for SlyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlyError::IoError(e) => write!(f, "Unexpected I/O error: {}", e),
            SlyError::SendError(e) => write!(f, "Channel closed unexpectedly: {}", e),
            SlyError::JoinError(e) => write!(f, "Error while joining: {}", e),
            SlyError::Generic(e) => write!(f, "Error encountered: {}", e),
        }
    }
}

impl std::error::Error for SlyError {}

impl<T> From<SendError<T>> for SlyError {
    fn from(err: SendError<T>) -> Self {
        let str = err.to_string();
        SlyError::SendError(str)
    }
}

impl From<JoinError> for SlyError {
    fn from(err: JoinError) -> Self {
        SlyError::JoinError(err)
    }
}

impl From<io::Error> for SlyError {
    fn from(inner: io::Error) -> Self {
        SlyError::IoError(inner)
    }
}
