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
                    if state.exit {
                        break;
                    }
                    if state.started {
                        tick_store._set_state(|state| state.increment_count());
                    }
                }
            }
        });
        Self { store }
    }

    pub fn store(&self) -> Arc<StateStore<CounterState>> {
        self.store.clone()
    }

    pub fn increment_count(&self) {
        self.store._set_state(|state| state.increment_count());
    }

    pub fn decrement_count(&self) {
        self.store._set_state(|state| state.decrement_count());
    }

    pub fn start_counter(&self) {
        self.store._set_state(|state| state.set_started(true));
    }

    pub fn stop_counter(&self) {
        self.store._set_state(|state| state.set_started(false));
    }

    pub fn reset_counter(&self) {
        self.store._set_state(|state| state.reset_count());
    }

    pub fn request_exit(&self) {
        self.store._set_state(|state| state.set_exit());
    }
}
