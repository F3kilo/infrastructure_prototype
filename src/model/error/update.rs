use super::model_not_free::ModelNotFreeError;
use std::error::Error;
use std::fmt;
use std::sync::mpsc::SendError;

#[derive(Debug)]
pub enum UpdateError {
    ModelNotFree(ModelNotFreeError),
    CantSendToPresenter,
}

impl Error for UpdateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UpdateError::ModelNotFree(e) => Some(e),
            UpdateError::CantSendToPresenter => None,
        }
    }
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.source() {
            Some(e) => fmt::Display::fmt(e, f),
            None => write!(f, "Error in model updating"),
        }
    }
}

impl<M> From<SendError<M>> for UpdateError {
    fn from(_: SendError<M>) -> Self {
        UpdateError::CantSendToPresenter
    }
}

impl From<ModelNotFreeError> for UpdateError {
    fn from(e: ModelNotFreeError) -> Self {
        UpdateError::ModelNotFree(e)
    }
}
