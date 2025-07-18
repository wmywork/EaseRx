use easerx::State;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CounterState {
    pub count: i32,
    pub started: bool,
    pub exit: bool,
}

impl State for CounterState {}

/*impl Default for CounterState {
    fn default() -> Self {
        Self {
            count: 0,
            started: false,
            exit: false,
        }
    }
}*/
impl CounterState {
    pub fn increment_count(self) -> Self {
        Self {
            count: self.count + 1,
            ..self
        }
    }

    pub fn decrement_count(self) -> Self {
        Self {
            count: self.count - 1,
            ..self
        }
    }

    pub fn reset_count(self) -> Self {
        Self { count: 0, ..self }
    }

    pub fn set_started(self, started: bool) -> Self {
        Self { started, ..self }
    }

    pub fn set_exit(self) -> Self {
        Self { exit: true, ..self }
    }
}
