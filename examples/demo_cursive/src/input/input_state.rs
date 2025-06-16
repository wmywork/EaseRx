use easerx::State;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct InputState {
    pub exit: bool,
    pub refresh_event: u8,
}

impl State for InputState {}

impl InputState {
   
    // Set exit flag
    pub fn set_exit(self) -> Self {
        Self { exit: true, ..self }
    }

} 