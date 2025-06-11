use easerx::State;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct InputState {
    pub exit: bool,
    pub draw_event: u8,
}

impl State for InputState {}

impl InputState {
    pub fn is_exit(&self) -> bool {
        self.exit
    }
    
    pub fn set_exit(self) -> Self {
        Self { exit: true, ..self }
    }

    pub fn send_draw_event(self) -> Self {
        let value = if self.draw_event == u8::MAX {
            0
        } else {
            self.draw_event + 1
        };
        Self {
            draw_event: value,
            ..self
        }
    }
}
