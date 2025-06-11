use crate::executor::executor_state::ExecutorState;
use easerx::StateStore;
use std::sync::Arc;
use std::time::Duration;

pub struct ExecutorModel {
    store: Arc<StateStore<ExecutorState>>,
}

impl ExecutorModel {
    pub fn new() -> Self {
        let store = Arc::new(StateStore::new(ExecutorState::default()));
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
                    if state.async_num.is_loading() {
                        tick_store._set_state(|state| state.on_tick());
                    }
                }
            }
        });
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
                store_set._set_state(|state| state.set_repeated_clicks(true));
                return;
            } else {
                store_set.execute(|| heavy_computation(), |state, num| state.set_async_num(num));
            }
        });
    }

    pub fn reset_num(&self) {
        self.store._set_state(|state| state.reset_num());
    }

    pub fn request_exit(&self) {
        self.store._set_state(|state| state.set_exit());
    }
}

fn heavy_computation() -> u64 {
    let mut i: u64 = 0;
    for _ in 0..400_000_000 {
        i = i + 1;
    }
    i
}
