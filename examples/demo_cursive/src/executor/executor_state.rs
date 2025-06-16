use easerx::{Async, State};

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutorState {
    pub async_num: Async<u64>,
    pub repeated_clicks: bool,
    pub exit: bool,
}

impl State for ExecutorState {}

impl Default for ExecutorState {
    fn default() -> Self {
        Self {
            async_num: Default::default(),
            repeated_clicks: false,
            exit: false,
        }
    }
}

impl ExecutorState {
    pub fn set_repeated_clicks(self, repeated: bool) -> Self {
        Self {
            repeated_clicks: repeated,
            ..self
        }
    }

    pub fn set_async_num(self, num: Async<u64>) -> Self {
        Self {
            async_num: num,
            repeated_clicks: false,
            ..self
        }
    }

    pub fn reset_num(self) -> Self {
        if self.async_num.is_complete() {
            Self {
                async_num: Async::default(),
                repeated_clicks: false,
                exit: self.exit,
            }
        } else {
            self
        }
    }
}
