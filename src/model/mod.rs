use crate::input::Input;

pub mod counter_model;
pub mod error;
pub mod model_manager;

pub trait Model {
    type PriorResult: Sized;

    fn prior(&self, prior_result: Option<Self::PriorResult>) -> Option<Self::PriorResult>;
    fn update(
        &mut self,
        prior_result: Option<Self::PriorResult>,
        inputs: impl Iterator<Item = Input>,
    ) -> Option<Self::PriorResult>;
}
