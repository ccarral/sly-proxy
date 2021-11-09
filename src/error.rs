use std::fmt;
use std::io;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug)]
pub enum FlyError<T> {
    IoError(std::io::Error),
    SendError(SendError<T>),
    Generic(String),
}

impl<T> fmt::Display for FlyError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlyError::IoError(e) => write!(f, "Unexpected I/O error: {}", e),
            FlyError::SendError(e) => write!(f, "Channel closed unexpectedly: {}", e),
            FlyError::Generic(e) => write!(f, "Error encountered: {}", e),
        }
    }
}

impl<T: fmt::Debug> std::error::Error for FlyError<T> {}

impl<T> From<SendError<T>> for FlyError<T> {
    fn from(err: SendError<T>) -> Self {
        FlyError::SendError(err)
    }
}

impl<T> From<io::Error> for FlyError<T> {
    fn from(inner: io::Error) -> Self {
        FlyError::IoError(inner)
    }
}
