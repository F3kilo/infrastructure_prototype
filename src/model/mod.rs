pub mod error;

use crate::input::Input;
use crate::model::error::model_not_free::ModelNotFreeError;
use crate::model::error::update::UpdateError;
use crate::model::error::ModelManagerError;
use crate::utils;
use crate::utils::wait_for;
use slog::{crit, error, info, debug, trace, warn, Logger};
use std::fmt;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use tinyfiledialogs;

#[derive(Debug)]
enum Command {
    Stop,
    Run,
    Exit,
}

#[derive(Debug)]
enum Notification {
    Error(ModelManagerError),
}

#[derive(Debug)]
enum State {
    Running,
    Stoped,
    Exitting,
}

#[derive(Debug)]
pub struct ServiceChannel {
    command_rx: Receiver<Command>,
    notifications_tx: Sender<Notification>,
}

#[derive(Debug)]
pub struct ModelManager<Model> {
    model: Arc<Model>,
    input_rx: Receiver<Input>,
    state: State,
    service_channel: ServiceChannel,
    model_tx: Sender<Arc<Model>>,
    logger: Logger,
}

impl<M: Model> ModelManager<M> {
    pub fn new(
        service_channel: ServiceChannel,
        model_tx: Sender<Arc<M>>,
        input_rx: Receiver<Input>,
        model: Arc<M>,
        logger: Logger,
    ) -> Self {
        trace!(logger, "Creating model manager");
        Self {
            model,
            input_rx,
            state: State::Running,
            service_channel,
            model_tx,
            logger,
        }
    }

    fn interpret_commands(&self) -> State {
        let mut state = State::Running;
        for command in self.service_channel.command_rx.try_iter() {
            trace!(self.logger, "Got command: {:?}", command);
            match command {
                Command::Stop => state = State::Stoped,
                Command::Run => state = State::Running,
                Command::Exit => return State::Exitting,
            }
        }
        state
    }

    fn try_get_mut_model(&mut self) -> Result<&mut M, ModelNotFreeError> {
        debug!(
            self.logger,
            "try_get_mut_model(&mut self) called. Strong reference count: {:?}",
            Arc::strong_count(&self.model)
        );
        match Arc::get_mut(&mut self.model) {
            Some(m) => Ok(m),
            None => {
                warn!(self.logger, "Arc::get_mut(...) failed",);
                Err(ModelNotFreeError::ArcGetMutFailed)
            }
        }
    }

    fn wait_for_mut_model(&mut self) -> Result<&mut M, ModelNotFreeError> {
        trace!(self.logger, "Start wait for &mut Model");
        let wait_result = wait_for(
            || Arc::strong_count(&self.model) == 1,
            Some(Duration::from_secs(2)),
            Duration::from_micros(500),
        );
        if wait_result {
            trace!(self.logger, "Got &mut Model");
            return self.try_get_mut_model();
        }
        warn!(self.logger, "Wait time for &mut Model exceeded");
        Err(ModelNotFreeError::WaitTimeoutExceeded)
    }

    fn take_input_events(&mut self) -> Vec<Input> {
        self.input_rx.try_iter().collect()
    }

    fn before_present(&mut self) -> Result<(), ModelNotFreeError> {
        trace!(self.logger, "Before present updating start");
        let input_events = self.take_input_events();
        trace!(self.logger, "Got {:?} input events", input_events.len());
        let model = self.wait_for_mut_model()?;
        model.before_present(input_events.into_iter());
        Ok(())
    }

    fn while_present(&self) {
        trace!(self.logger, "While present calculations start");
        self.model.while_present();
    }

    fn after_present(&mut self) -> Result<(), ModelNotFreeError> {
        trace!(self.logger, "After present updating start");
        let model = self.wait_for_mut_model()?;
        model.after_present();
        Ok(())
    }

    fn update(&mut self) -> Result<(), UpdateError> {
        self.before_present()?;
        trace!(
            self.logger,
            "Before present updating done. Sending Arc clone..."
        );
        self.model_tx.send(self.model.clone())?;
        trace!(self.logger, "Arc clone sent");
        self.while_present();
        trace!(self.logger, "While present updating done");
        self.after_present()?;
        trace!(self.logger, "After present updating done");
        Ok(())
    }

    pub fn run(mut self) {
        self.state = State::Running;
        trace!(self.logger, "Starting model manager loop");
        loop {
            self.state = self.interpret_commands();
            trace!(self.logger, "New state is: {:?}", self.state);

            match self.state {
                State::Running => {
                    trace!(self.logger, "State::Running: Start updating model");
                    self.update().unwrap_or_else(|e| {
                        error!(self.logger, "Update error: {}", e);
                        trace!(self.logger, "Stop model manager loop");
                        self.state = State::Stoped;
                        trace!(self.logger, "Sending error");
                        self.send_error(e.into());
                    });
                }
                State::Stoped => {
                    trace!(self.logger, "State::Stopped: waiting for next commands");
                    self.wait_for_next_commands();
                }
                State::Exitting => {
                    trace!(self.logger, "State::Exitting: leaving loop");
                    break;
                }
            }
        }
    }

    fn wait_for_next_commands(&mut self) {
        trace!(self.logger, "Start waiting for commands");
        let command = self.service_channel.command_rx.recv();
        match command {
            Ok(c) => {
                trace!(self.logger, "Command recieved: {:?}", c);
                self.state = match c {
                    Command::Run => State::Running,
                    Command::Exit => State::Exitting,
                    _ => State::Stoped,
                }
            }
            Err(e) => {
                trace!(self.logger, "Command recieve error: {}", e);
                self.send_error(e.into())
            }
        }
    }

    fn send_error(&mut self, e: ModelManagerError) {
        trace!(self.logger, "Sending error: {}", e);
        self.service_channel
            .notifications_tx
            .send(Notification::Error(e))
            .unwrap_or_else(|e| {
                let error_message = format!("Can't send notification to main: {}", e);
                utils::show_error_message("Model manager error", error_message.as_str());
                error!(self.logger, "{}", error_message);
                self.state = State::Exitting;
            });
    }
}

pub trait Model {
    fn before_present(&mut self, inputs: impl Iterator<Item = Input>);
    fn while_present(&self);
    fn after_present(&mut self);
}
