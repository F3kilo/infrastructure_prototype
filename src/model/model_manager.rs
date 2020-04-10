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

/// Commands from outer code.
#[derive(Debug)]
pub enum Command {
    /// Stop model updating loop and wait for next commands.
    Stop,
    /// Run model updating loop.
    Run,
    /// Exit from model updating loop.
    Exit,
}

/// Notifications to outer code
#[derive(Debug)]
pub enum Notification {
    Error(ModelManagerError),
}

/// Current model updating loop state
#[derive(Debug)]
enum State {
    Running,
    Stoped,
    Exitting,
}

/// Manager, that controll model by calling it's trait methods in correct order, at correct time.
#[derive(Debug)]
pub struct ModelManager<M> {
    model: Arc<M>,
    inner_bonds: InnerBonds<M>,
    state: State,
    logger: Logger,
}

/// Outer code communicate with ModelManager using this.
#[derive(Debug)]
pub struct OuterBonds<M> {
    pub input_tx: Sender<Input>,
    pub command_tx: Sender<Command>,
    pub notification_rx: Receiver<Notification>,
    pub model_rx: Receiver<Arc<M>>,
}

/// ModelManager communicate with outer code using this.
#[derive(Debug)]
struct InnerBonds<M> {
    input_rx: Receiver<Input>,
    command_rx: Receiver<Command>,
    notification_tx: Sender<Notification>,
    model_tx: Sender<Arc<M>>,
}

impl<M: Model> ModelManager<M> {
    pub fn new(model: Arc<M>, logger: Logger) -> (Self, OuterBonds<M>) {
        let (outer_bonds, inner_bonds) = <ModelManager<M>>::create_bonds();

        trace!(logger, "Creating model manager");
        let model_manager = Self {
            model,
            inner_bonds,
            state: State::Stoped,
            logger,
        };
        (model_manager, outer_bonds)
    }

    /// Creates inner and outer parts of communication tools
    fn create_bonds() -> (OuterBonds<M>, InnerBonds<M>) {
        let (command_tx, command_rx) = channel();
        let (notification_tx, notification_rx) = channel();
        let (input_tx, input_rx) = channel();
        let (model_tx, model_rx) = channel();

        let outer_bonds = OuterBonds {
            input_tx,
            command_tx,
            notification_rx,
            model_rx,
        };

        let inner_bonds = InnerBonds {
            input_rx,
            command_rx,
            notification_tx,
            model_tx,
        };
        (outer_bonds, inner_bonds)
    }

    /// Interpret recieved commands and return state required by them.
    fn interpret_commands(&self) -> State {
        let mut state = State::Running;
        for command in self.inner_bonds.command_rx.try_iter() {
            trace!(self.logger, "Got command: {:?}", command);
            match command {
                Command::Stop => state = State::Stoped,
                Command::Run => state = State::Running,
                Command::Exit => return State::Exitting,
            }
        }
        state
    }

    /// Tries to get mutable reference from Arc to model.
    /// If count of strong references in that Arc is bigger than 1, then returns ModelNotFreeError::ArcGetMutFailed.
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

    /// Waits for strong reference count in Arc to model equals to 1.
    /// Return ModelNotFreeError::WaitTimeoutExceeded if wait time more than timeout.
    /// Return ModelNotFreeError::ArcGetMutFailed if whatever can't get mut reference by some reason.
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

    /// Clone recieved events into Vec.
    fn take_input_events(&mut self) -> Vec<Input> {
        self.inner_bonds.input_rx.try_iter().collect()
    }

    /// Updates model before send to presenter.
    fn before_present(
        &mut self,
        prior_result: Option<M::PriorResult>,
    ) -> Result<Option<M::PriorResult>, ModelNotFreeError> {
        trace!(self.logger, "Before present updating start");
        let input_events = self.take_input_events();
        trace!(self.logger, "Got {:?} input events", input_events.len());
        let model = self.wait_for_mut_model()?;
        Ok(model.update(prior_result, input_events.into_iter()))
    }

    /// Makes some calculations in model while it is shared with presenter.
    fn while_present(&self, prior_result: Option<M::PriorResult>) -> Option<M::PriorResult> {
        trace!(self.logger, "While present calculations start");
        self.model.prior(prior_result)
    }

    /// Updates model and sends it to presenter
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
        self.inner_bonds.model_tx.send(self.model.clone())?;
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
        let command = self.inner_bonds.command_rx.recv();
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
        self.inner_bonds
            .notification_tx
            .send(Notification::Error(e))
            .unwrap_or_else(|e| {
                let error_message = format!("Can't send notification to main: {}", e);
                utils::show_error_message("Model manager error", error_message.as_str());
                error!(self.logger, "{}", error_message);
                self.state = State::Exitting;
            });
    }
}
