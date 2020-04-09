use crate::input::Input;
use slog::{error, Logger};
use std::sync::mpsc::Receiver;

/// Logs all input from InputController
pub struct InputLogger {
    rx: Receiver<Input>,
    logger: Logger,
}

impl InputLogger {
    pub fn new(rx: Receiver<Input>, logger: Logger) -> Self {
        Self { rx, logger }
    }

    pub fn run(self) {
        loop {
            match self.rx.recv() {
                Ok(input) => {
                    println!("Got input: {:?}", input);
                }
                Err(e) => error!(self.logger, "Can't recieve input: {}", e),
            }
        }
    }
}
