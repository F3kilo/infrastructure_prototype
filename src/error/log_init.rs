use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum LogInitError {
    Slog(sloggers::Error),
    Io(std::io::Error),
}

impl std::error::Error for LogInitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LogInitError::Io(e) => Some(e),
            LogInitError::Slog(e) => Some(e),
        }
    }
}

impl Display for LogInitError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.source().unwrap(), f)
    }
}

impl From<sloggers::Error> for LogInitError {
    fn from(e: sloggers::Error) -> Self {
        Self::Slog(e)
    }
}

impl From<std::io::Error> for LogInitError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
