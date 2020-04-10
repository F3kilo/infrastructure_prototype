use crate::input::Input;

pub mod counter_model;
pub mod error;
pub mod model_manager;

pub enum State<Result> {
    Running(Option<Result>),
    Finished
}

pub trait Model {
    type PriorResult: Sized;

    fn prior(&self, prior_result: Option<Self::PriorResult>) -> State<Self::PriorResult>;
    fn update(
        &mut self,
        prior_result: Option<Self::PriorResult>,
        inputs: impl Iterator<Item = Input>,
    ) -> State<Self::PriorResult>;
}
