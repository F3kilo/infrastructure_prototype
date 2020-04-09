use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ModelNotFreeError {
    WaitTimeoutExceeded,
    ArcGetMutFailed,
}

impl Error for ModelNotFreeError {}

impl fmt::Display for ModelNotFreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.source() {
            Some(e) => fmt::Display::fmt(e, f),
            None => write!(f, "Model was not free when model manager expect"),
        }
    }
}
