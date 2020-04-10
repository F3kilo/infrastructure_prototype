#![allow(dead_code)]

mod error;
mod input;
mod input_logger;
mod model;
mod presenter;
mod renderer;
mod utils;

use winit::dpi::PhysicalSize;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use settings_path::*;
use slog::{info, trace, warn, Logger};
use sloggers::{file::FileLoggerBuilder, types::TimeZone, Build};
use std::path::PathBuf;

use error::init::InitError;
use error::log_init::LogInitError;
use sloggers::types::Severity;
use winit::error::OsError;

use crate::input::Input;
use crate::model::counter_model::CounterModel;
use crate::model::model_manager::{ModelManager, Command};
use crate::utils::show_error_message;
use std::convert::TryInto;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use std::time::Duration;

fn main() {
    let (logger, event_loop, _window, input_tx, model_rx) = init().unwrap_or_else(|e| {
        let message = format!("Initialization error occurred: {}", e);
        show_error_message("Initialization error", message.as_str());
        panic!(message);
    });

    std::thread::spawn(move || {
       loop {
           if let Ok(model) = model_rx.try_recv() {
               println!("Got model. Count: {:?}", model.count());
           }
           std::thread::sleep(Duration::from_millis(16));
       }
    });

    info!(logger, "Initialization done");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(input) = (&event).try_into() {
            input_tx.send(input).unwrap_or_else(|e| {
                warn!(logger, "Can't send input event, because: {}", e);
            });
            return;
        }

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            info!(logger, "Exiting...");
            *control_flow = ControlFlow::Exit;
        }
    });
}

/// Basis structures initialization
fn init() -> Result<(Logger, EventLoop<()>, Window, Sender<Input>, Receiver<Arc<CounterModel>>), InitError> {
    let mut save_path = default_settings_path()?;
    save_path.push("InfrastructurePrototype");

    // Init logger
    let logger = init_logger(save_path)?;
    info!(logger, "=============== START NEW SESSION ===============");
    trace!(logger, "Logger initilized");

    // Init event loop
    let event_loop = EventLoop::new();
    trace!(logger, "Event loop initialized");

    // Init window
    let window = init_window(&event_loop)?;
    trace!(logger, "Window initialized");

    // Init input printer
    // let (tx, rx) = channel();
    // let input_logger = InputLogger::new(rx, logger.clone());
    // std::thread::spawn(|| input_logger.run());

    let (model_manager, input_tx, commands_tx, _, model_rx) =
        ModelManager::new(Arc::new(CounterModel::new()), logger.clone());

    std::thread::spawn(|| model_manager.run());

    commands_tx.send(Command::Run).unwrap();

    Ok((logger, event_loop, window, input_tx, model_rx))
}

/// Window initialization
fn init_window(event_loop: &EventLoop<()>) -> Result<Window, OsError> {
    let window_builder = WindowBuilder::default()
        .with_title("InfrastructurePrototype")
        .with_inner_size(PhysicalSize::new(800, 600));
    let window = window_builder.build(&event_loop)?;
    Ok(window)
}

/// Logger initialization
fn init_logger(save_path: PathBuf) -> Result<Logger, LogInitError> {
    let log_dir = save_path.join("logs");
    let log_path = save_path.join("logs\\log");
    std::fs::create_dir_all(log_dir)?;
    let logger = FileLoggerBuilder::new(log_path)
        .timezone(TimeZone::Local)
        .rotate_size(10 * 2u64.pow(20))
        .level(Severity::Trace)
        .build()?;
    Ok(logger)
}
