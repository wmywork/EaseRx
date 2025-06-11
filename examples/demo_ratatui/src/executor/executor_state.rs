use easerx::{Async, State};
use throbber_widgets_tui::ThrobberState;

#[derive(Clone, Debug)]
pub struct ExecutorState {
    pub async_num: Async<u64>,
    pub repeated_clicks: bool,
    pub throbber_state: ThrobberState,
    pub exit: bool,
}

impl State for ExecutorState {}

impl Default for ExecutorState {
    fn default() -> Self {
        Self {
            async_num: Default::default(),
            repeated_clicks: false,
            throbber_state: Default::default(),
            exit: false,
        }
    }
}
impl ExecutorState {
    pub fn on_tick(self) -> Self {
        let mut new_throbber_state = self.throbber_state.clone();
        new_throbber_state.calc_next();
        Self {
            throbber_state: new_throbber_state,
            ..self
        }
    }
    pub fn set_exit(self) -> Self {
        Self { exit: true, ..self }
    }

    pub fn reset_num(self) -> Self {
        if self.async_num.is_complete() {
            Self {
                async_num: Async::default(),
                repeated_clicks: false,
                throbber_state: Default::default(),
                exit: self.exit,
            }
        } else {
            self
        }
    }
}
