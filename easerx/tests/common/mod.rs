use easerx::{Async, State};

#[derive(Clone, Debug, PartialEq)]
pub struct TestState {
    pub num: Async<u64>,
}

impl State for TestState {}

impl Default for TestState {
    fn default() -> Self {
        TestState {
            num: Async::Uninitialized,
        }
    }
}

impl TestState {
    pub fn set_num(self, async_data: Async<u64>) -> Self {
        Self {
            num: async_data,
            ..self
        }
    }
}