use std::fmt;
use std::io;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug)]
pub enum FlyError {
    IoError(std::io::Error),
    SendError(String),
    Generic(String),
}

impl fmt::Display for FlyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlyError::IoError(e) => write!(f, "Unexpected I/O error: {}", e),
            FlyError::SendError(e) => write!(f, "Channel closed unexpectedly: {}", e),
            FlyError::Generic(e) => write!(f, "Error encountered: {}", e),
        }
    }
}

impl std::error::Error for FlyError {}

impl<T> From<SendError<T>> for FlyError {
    fn from(err: SendError<T>) -> Self {
        let str = err.to_string();
        FlyError::SendError(str)
    }
}

impl From<io::Error> for FlyError {
    fn from(inner: io::Error) -> Self {
        FlyError::IoError(inner)
    }
}
