use crate::counter::counter_state::CounterState;
use easerx::StateStore;
use std::sync::Arc;
use std::time::Duration;
pub struct CounterViewModel {
    store: Arc<StateStore<CounterState>>,
}

impl CounterViewModel {
    pub fn new() -> Self {
        let store = Arc::new(StateStore::new(CounterState::default()));
        let mut tick_interval = {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            interval
        };
        let tick_store = store.clone();
        tokio::spawn(async move {
            loop {
                tick_interval.tick().await;
                if let Ok(state) = tick_store.await_state().await {
                    if state.started {
                        tick_store._set_state(|state| state.on_counter_increment());
                    }
                }
            }
        });
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
