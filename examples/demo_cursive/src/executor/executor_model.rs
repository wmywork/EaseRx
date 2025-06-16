use crate::executor::executor_state::ExecutorState;
use easerx::StateStore;
use std::sync::Arc;

pub struct ExecutorModel {
    store: Arc<StateStore<ExecutorState>>,
}

impl ExecutorModel {
    // Create a new ExecutorModel with default state and start the background task
    pub fn new() -> Self {
        let store = Arc::new(StateStore::new(ExecutorState::default()));
        Self { store }
    }

    // Get a clone of the state store
    pub fn store(&self) -> Arc<StateStore<ExecutorState>> {
        self.store.clone()
    }

    // Request calculation (async operation)
    pub fn request_calc(&self) {
        let store_set = self.store.clone();
        self.store._with_state(move |state| {
            if state.async_num.is_loading() {
                //show repeated clicks and return
                store_set._set_state(|state| state.set_repeated_clicks(true));
                return;
            } else {
                store_set.execute(
                    || heavy_computation(),
                    |state, num| state.set_async_num(num),
                );
            }
        });
    }

    pub fn reset_num(&self) {
        self.store._set_state(|state| state.reset_num());
    }
}

fn heavy_computation() -> u64 {
    let mut i: u64 = 0;
    for _ in 0..400_000_000 {
        i = i + 1;
    }
    i
}
