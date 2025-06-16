use crate::counter::counter_state::CounterState;
use easerx::StateStore;
use std::sync::Arc;
use std::time::Duration;

pub struct CounterModel {
    store: Arc<StateStore<CounterState>>,
}

impl CounterModel {
    // Create a new CounterViewModel with default state and start the background task
    pub fn new() -> Self {
        let store = Arc::new(StateStore::new(CounterState::default()));
        
        // Setup a ticker that will increment the counter when started
        let mut tick_interval = {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            interval
        };
        
        // Clone store for the async task
        let tick_store = store.clone();
        
        // Spawn background task to handle automatic counting
        tokio::spawn(async move {
            loop {
                tick_interval.tick().await;
                if let Ok(state) = tick_store.await_state().await {
                    // Check if we should exit
                    if state.exit {
                        break;
                    }
                    // If counter is started, increment it
                    if state.started {
                        tick_store._set_state(|state| state.increment_count());
                    }
                }
            }
        });
        
        Self { store }
    }

    // Get a clone of the state store
    pub fn store(&self) -> Arc<StateStore<CounterState>> {
        self.store.clone()
    }

    // Manually increment counter
    pub fn increment_count(&self) {
        self.store._set_state(|state| state.increment_count());
    }

    // Manually decrement counter
    pub fn decrement_count(&self) {
        self.store._set_state(|state| state.decrement_count());
    }

    // Start automatic counting
    pub fn start_counter(&self) {
        self.store._set_state(|state| state.set_started(true));
    }

    // Stop automatic counting
    pub fn stop_counter(&self) {
        self.store._set_state(|state| state.set_started(false));
    }

    // Reset counter to 0
    pub fn reset_counter(&self) {
        self.store._set_state(|state| state.reset_count());
    }

    // Signal exit to stop background task
    pub fn request_exit(&self) {
        self.store._set_state(|state| state.set_exit());
    }
    
} 