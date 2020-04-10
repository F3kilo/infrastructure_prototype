use crate::input::Input;

pub mod error;
pub mod model_manager;

pub trait Model {
    fn before_present(&mut self, inputs: impl Iterator<Item = Input>);
    fn while_present(&self);
    fn after_present(&mut self);
}
