use super::log_init::LogInitError;
use settings_path::FindPathError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use winit::error::OsError;

#[derive(Debug)]
pub enum InitError {
    Path(FindPathError),
    Log(LogInitError),
    Os(OsError),
}

impl std::error::Error for InitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InitError::Path(e) => Some(e),
            InitError::Log(e) => Some(e),
            InitError::Os(e) => Some(e),
        }
    }
}

impl Display for InitError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.source().unwrap(), f)
    }
}

impl From<LogInitError> for InitError {
    fn from(e: LogInitError) -> Self {
        Self::Log(e)
    }
}

impl From<FindPathError> for InitError {
    fn from(e: FindPathError) -> Self {
        Self::Path(e)
    }
}

impl From<OsError> for InitError {
    fn from(e: OsError) -> Self {
        Self::Os(e)
    }
}
