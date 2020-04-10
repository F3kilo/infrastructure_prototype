use crate::input::Input;
use crate::model::error::model_not_free::ModelNotFreeError;
use crate::model::error::update::UpdateError;
use crate::model::error::ModelManagerError;
use crate::model::Model;
use crate::utils;
use crate::utils::wait_for;
use slog::{debug, error, trace, warn, Logger};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub enum Command {
    Stop,
    Run,
    Exit,
}

#[derive(Debug)]
pub enum Notification {
    Error(ModelManagerError),
}

#[derive(Debug)]
enum State {
    Running,
    Stoped,
    Exitting,
}

#[derive(Debug)]
struct ServiceChannel {
    command_rx: Receiver<Command>,
    notifications_tx: Sender<Notification>,
}

#[derive(Debug)]
pub struct ModelManager<Model> {
    model: Arc<Model>,
    input_rx: Receiver<Input>,
    state: State,
    commands_rx: Receiver<Command>,
    notifications_tx: Sender<Notification>,
    model_tx: Sender<Arc<Model>>,
    logger: Logger,
}

impl<M: Model> ModelManager<M> {
    pub fn new(
        model: Arc<M>,
        logger: Logger,
    ) -> (
        Self,
        Sender<Input>,
        Sender<Command>,
        Receiver<Notification>,
        Receiver<Arc<M>>,
    ) {
        let (commands_tx, commands_rx) = channel();
        let (notifications_tx, notifications_rx) = channel();
        let (input_tx, input_rx) = channel();
        let (model_tx, model_rx) = channel();

        trace!(logger, "Creating model manager");
        let s = Self {
            model,
            input_rx,
            state: State::Stoped,
            commands_rx,
            notifications_tx,
            model_tx,
            logger,
        };
        (s, input_tx, commands_tx, notifications_rx, model_rx)
    }

    fn interpret_commands(&self) -> State {
        let mut state = State::Running;
        for command in self.commands_rx.try_iter() {
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

    fn before_present(&mut self, prior_result: Option<M::PriorResult>) -> Result<Option<M::PriorResult>, ModelNotFreeError> {
        trace!(self.logger, "Before present updating start");
        let input_events = self.take_input_events();
        trace!(self.logger, "Got {:?} input events", input_events.len());
        let model = self.wait_for_mut_model()?;
        Ok(model.update(prior_result, input_events.into_iter()))
    }

    fn while_present(&self, prior_result: Option<M::PriorResult>) -> Option<M::PriorResult> {
        trace!(self.logger, "While present calculations start");
        self.model.prior(prior_result)
    }

    fn update(
        &mut self,
        prior_result: Option<M::PriorResult>,
    ) -> Result<Option<M::PriorResult>, UpdateError> {
        let prior_result = self.while_present(prior_result);
        trace!(self.logger, "While present calculations done");

        let prior_result = self.before_present(prior_result)?;
        trace!(
            self.logger,
            "Before present updating done. Sending Arc clone..."
        );
        self.model_tx.send(self.model.clone())?;
        trace!(self.logger, "Arc clone sent");

        Ok(prior_result)
    }

    pub fn run(mut self) {
        trace!(self.logger, "Starting model manager loop");
        let mut prior_result = None;
        loop {
            self.state = self.interpret_commands();
            trace!(self.logger, "New state is: {:?}", self.state);

            match self.state {
                State::Running => {
                    trace!(self.logger, "State::Running: Start updating model");
                    prior_result = self.update(prior_result).unwrap_or_else(|e| {
                        error!(self.logger, "Update error: {}", e);
                        trace!(self.logger, "Stop model manager loop");
                        self.state = State::Stoped;
                        trace!(self.logger, "Sending error");
                        self.send_error(e.into());
                        None
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
        let command = self.commands_rx.recv();
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
        self.notifications_tx
            .send(Notification::Error(e))
            .unwrap_or_else(|e| {
                let error_message = format!("Can't send notification to main: {}", e);
                utils::show_error_message("Model manager error", error_message.as_str());
                error!(self.logger, "{}", error_message);
                self.state = State::Exitting;
            });
    }
}
