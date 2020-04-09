pub mod model_not_free;
pub mod update;

use std::error::Error;
use std::fmt;
use std::sync::mpsc::RecvError;
use update::UpdateError;

#[derive(Debug)]
pub enum ModelManagerError {
    UpdateError(UpdateError),
    RecvError(RecvError),
}

impl Error for ModelManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ModelManagerError::UpdateError(e) => Some(e),
            ModelManagerError::RecvError(e) => Some(e),
        }
    }
}

impl fmt::Display for ModelManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.source().unwrap(), f)
    }
}

impl From<UpdateError> for ModelManagerError {
    fn from(e: UpdateError) -> Self {
        ModelManagerError::UpdateError(e)
    }
}

impl From<RecvError> for ModelManagerError {
    fn from(e: RecvError) -> Self {
        ModelManagerError::RecvError(e)
    }
}
