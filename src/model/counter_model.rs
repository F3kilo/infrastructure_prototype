use crate::input::{Input, InputEvent};
use winit::event::{ElementState, VirtualKeyCode};
use super::{State, Model};

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
    type PriorResult = ();

    fn prior(&self, _: Option<Self::PriorResult>) -> State<Self::PriorResult> {
        State::Running(None)
    }

    fn update(
        &mut self,
        _: Option<Self::PriorResult>,
        inputs: impl Iterator<Item = Input>,
    ) -> State<Self::PriorResult> {
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
        State::Running(None)
    }
}
