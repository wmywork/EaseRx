use crate::counter::counter_state::CounterState;
use easerx::StateStore;
use std::sync::Arc;

pub struct CounterViewModel {
    store: Arc<StateStore<CounterState>>,
}

impl CounterViewModel {
    pub fn new() -> Self {
        let store = Arc::new(StateStore::new(CounterState::default()));
        Self { store }
    }

    pub fn store(&self) -> Arc<StateStore<CounterState>> {
        self.store.clone()
    }

    pub fn increment_counter(&self) {
        self.store._set_state(|state| CounterState {
            count: state.count + 1,
            ..state
        });
    }

    pub fn decrement_counter(&self) {
        self.store._set_state(|state| CounterState {
            count: state.count - 1,
            ..state
        });
    }

    pub fn start_counter(&self) {
        self.store._set_state(|state| CounterState {
            started: true,
            ..state
        });
    }

    pub fn stop_counter(&self) {
        self.store._set_state(|state| CounterState {
            started: false,
            ..state
        });
    }

    pub fn reset_counter(&self) {
        self.store
            ._set_state(|state| CounterState { count: 0, ..state });
    }
}
