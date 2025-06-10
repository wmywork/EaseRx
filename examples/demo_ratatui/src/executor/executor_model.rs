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
            let mut interval = tokio::time::interval(Duration::from_millis(250));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            interval
        };
        let tick_store = store.clone();
        tokio::spawn(async move {
            loop {
                tick_interval.tick().await;
                if let Ok(state) = tick_store.await_state().await {
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

fn _fibonacci(n: u64) -> u64 {
    let (mut a, mut b) = (0, 1);
    for _ in 0..n {
        let next = a + b;
        a = b;
        b = next;
    }
    a
}

fn _fibonacci_result(n: u64) -> Result<u64, String> {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        match a.checked_add(b) {
            Some(next) => {
                a = b;
                b = next;
            }
            None => return Err(format!("Fibonacci calculation overflow at n={}", n)),
        }
    }
    Ok(a)
}

fn _fibonacci_option(n: u64) -> Option<u64> {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        match a.checked_add(b) {
            Some(next) => {
                a = b;
                b = next;
            }
            None => return None, // 溢出检测
        }
    }
    Some(a)
}

fn heavy_computation() -> u64 {
    let mut i: u64 = 0;
    for _ in 0..400_000_000 {
        i = i + 1;
    }
    i
}
