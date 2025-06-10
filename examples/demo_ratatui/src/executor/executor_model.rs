use crate::executor::executor_state::ExecutorState;
use easerx::StateStore;
use std::sync::Arc;

pub struct ExecutorModel {
    store: Arc<StateStore<ExecutorState>>,
}

impl ExecutorModel {
    pub fn new() -> Self {
        let store = Arc::new(StateStore::new(ExecutorState::default()));
        Self { store }
    }

    pub fn store(&self) -> Arc<StateStore<ExecutorState>> {
        self.store.clone()
    }

    pub fn request_calc(&self) {
        let store_set = self.store.clone();
        self.store._with_state(move |state| {
            if state.async_num.is_loading() {
                //show repeated clicks and return
                store_set._set_state(|s| ExecutorState {
                    repeated_clicks: true,
                    ..s
                });
                return;
            } else {
                store_set.execute(
                    || heavy_computation(),
                    |last, num| ExecutorState {
                        async_num: num,
                        repeated_clicks: false,
                        ..last
                    },
                );
            }
        });
    }
}

fn heavy_computation() -> u64 {
    let mut i: u64 = 0;
    for _ in 0..400_000_000 {
        i = i + 1;
    }
    i
}
