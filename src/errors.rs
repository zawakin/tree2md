use std::io;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    Io(io::Error),
    Stdin(crate::input::StdinError),
    InvalidPattern(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Stdin(e) => write!(f, "Stdin error: {}", e),
            AppError::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<crate::input::StdinError> for AppError {
    fn from(e: crate::input::StdinError) -> Self {
        AppError::Stdin(e)
    }
}