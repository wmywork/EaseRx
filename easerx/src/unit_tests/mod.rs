use crate::{Async, State};

// Import test modules
mod async_state_test;
mod execution_result_test;

mod async_executes_test;
mod execute_test;
mod state_store_test;
mod stream_ext_test;

#[derive(Clone, Debug, PartialEq)]
pub struct TestState {
    pub data: Async<String>,
}

impl State for TestState {}

impl Default for TestState {
    fn default() -> Self {
        TestState {
            data: Async::Uninitialized,
        }
    }
}

impl TestState {
    pub fn set_async_data(self, async_data: Async<String>) -> Self {
        Self {
            data: async_data,
            ..self
        }
    }
}