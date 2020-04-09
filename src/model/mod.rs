use crate::input::{Input, InputEvent};
use std::collections::{HashMap, LinkedList};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use winit::event::ElementState;
use winit::event::VirtualKeyCode;

pub enum State {
    Menu,
    Editor,
}

pub struct NumberState {
    value: i32,
    up_state: ElementState,
    down_state: ElementState,
}

impl NumberState {
    pub fn new() -> Self {
        Self {
            value: 0,
            up_state: ElementState::Released,
            down_state: ElementState::Released,
        }
    }

    fn up_clicked(&self, new_state: ElementState) -> bool {
        new_state == ElementState::Pressed && self.up_state == ElementState::Released
    }

    fn down_clicked(&self, new_state: ElementState) -> bool {
        new_state == ElementState::Pressed && self.down_state == ElementState::Released
    }

    fn update(&mut self, key: VirtualKeyCode, state: ElementState) {
        match key {
            VirtualKeyCode::Up => {
                if self.up_clicked(state) {
                    self.value += 1;
                }
                self.up_state = state;
            }
            VirtualKeyCode::Down => {
                if self.down_clicked(state) {
                    self.value -= 1;
                }
                self.up_state = state;
            }
            _ => {}
        };
    }
}

pub enum ControlEvents {
    Start,
    Stop,
    Pause,
}

pub struct HexFieldPlaygroundModel {
    input_rx: Receiver<Input>,
    control_rx: Receiver<ControlEvents>,
    number_state: Arc<NumberState>,
    number_state_tx: Sender<Arc<NumberState>>,
}

impl HexFieldPlaygroundModel {
    pub fn new(
        input_rx: Receiver<Input>,
        number_state_tx: Sender<Arc<NumberState>>,
        control_rx: Receiver<ControlEvents>,
    ) -> Self {
        let number_state = Arc::new(NumberState::new());
        Self {
            input_rx,
            control_rx,
            number_state,
            number_state_tx,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(num_state) = Arc::get_mut(&mut self.number_state) {}
        }
    }
}
