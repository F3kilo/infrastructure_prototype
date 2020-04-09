use std::convert::TryFrom;
use std::time::Instant;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};

#[derive(Debug, Clone)]
pub struct Input {
    happen_at: Instant,
    device_id: Option<DeviceId>,
    event: InputEvent,
}

impl From<(Option<DeviceId>, InputEvent)> for Input {
    fn from(input: (Option<DeviceId>, InputEvent)) -> Self {
        Self {
            happen_at: Instant::now(),
            device_id: input.0,
            event: input.1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Keyboard {
        key: VirtualKeyCode,
        state: ElementState,
    },
    RawMouseMove {
        delta: (f64, f64),
    },
    CursorMove {
        position: (f64, f64),
    },
    Scroll {
        delta: (f64, f64),
    },
    MouseButton {
        button: MouseButton,
        state: ElementState,
    },
    Symbol(char),
}

impl From<MouseScrollDelta> for InputEvent {
    fn from(delta: MouseScrollDelta) -> Self {
        let delta = match delta {
            MouseScrollDelta::PixelDelta(pos) => pos.into(),
            MouseScrollDelta::LineDelta(x, y) => (x as f64, y as f64),
        };
        InputEvent::Scroll { delta }
    }
}

impl From<(MouseButton, ElementState)> for InputEvent {
    fn from(input: (MouseButton, ElementState)) -> Self {
        InputEvent::MouseButton {
            button: input.0,
            state: input.1,
        }
    }
}

impl TryFrom<&Event<'_, ()>> for Input {
    type Error = ();

    fn try_from(event: &Event<()>) -> Result<Self, Self::Error> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::ReceivedCharacter(c) => {
                    let event = InputEvent::Symbol(*c);
                    Ok((None, event).into())
                }
                WindowEvent::KeyboardInput {
                    input, device_id, ..
                } => {
                    if let Some(key) = input.virtual_keycode {
                        let event = InputEvent::Keyboard {
                            key,
                            state: input.state,
                        };
                        return Ok((Some(*device_id), event).into());
                    }
                    Err(())
                }
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                    ..
                } => {
                    let event = InputEvent::CursorMove {
                        position: (*position).into(),
                    };
                    Ok((Some(*device_id), event).into())
                }
                WindowEvent::MouseWheel {
                    device_id, delta, ..
                } => {
                    let event = (*delta).into();
                    Ok((Some(*device_id), event).into())
                }
                WindowEvent::MouseInput {
                    device_id,
                    button,
                    state,
                    ..
                } => {
                    let event = InputEvent::MouseButton {
                        button: *button,
                        state: *state,
                    };
                    Ok((Some(*device_id), event).into())
                }
                _ => Err(()),
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                device_id,
            } => {
                let event = InputEvent::RawMouseMove { delta: *delta };
                Ok((Some(*device_id), event).into())
            }
            _ => Err(()),
        }
    }
}
