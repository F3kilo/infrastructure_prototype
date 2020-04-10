use super::Model;
use crate::input::{Input, InputEvent};
use winit::event::{ElementState, VirtualKeyCode};

pub struct CounterModel {
    counter: i32,
    up_state: ElementState,
    down_state: ElementState,
}

impl CounterModel {
    pub fn new() -> Self {
        Self {
            counter: 0,
            up_state: ElementState::Released,
            down_state: ElementState::Released,
        }
    }

    fn up_event(&mut self, new_state: ElementState) {
        if self.up_state == ElementState::Released && new_state == ElementState::Pressed {
            self.counter += 1;
        }
        self.up_state = new_state;
    }

    fn down_event(&mut self, new_state: ElementState) {
        if self.down_state == ElementState::Released && new_state == ElementState::Pressed {
            self.counter -= 1;
        }
        self.down_state = new_state;
    }

    pub fn count(&self) -> i32 {
        self.counter
    }
}

impl Model for CounterModel {
    fn before_present(&mut self, inputs: impl Iterator<Item = Input>) {
        for input in inputs {
            if let InputEvent::Keyboard { key, state } = input.event() {
                if *key == VirtualKeyCode::Up {
                    self.up_event(*state)
                }
                if *key == VirtualKeyCode::Down {
                    self.down_event(*state)
                }
            }
        }
    }

    fn while_present(&self) {}

    fn after_present(&mut self) {}
}
