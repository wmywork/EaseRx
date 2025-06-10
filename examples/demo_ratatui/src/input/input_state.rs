use easerx::State;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct InputState {
    pub exit: bool,
}

impl State for InputState {}
