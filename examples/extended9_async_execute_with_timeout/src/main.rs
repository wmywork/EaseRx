use crate::tracing_setup::tracing_init;
use easerx::{Async, State, StateStore};
use futures_signals::signal::SignalExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};
mod tracing_setup;

#[derive(Debug, Clone, Default)]
struct Counter {
    num: Async<u64>,
}

impl State for Counter {}

#[tokio::main]
async fn main() {
    tracing_init();

    info!("==========================================");
    warn!("A. Async execution will complete successfully");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.async_execute_with_timeout(
            async { heavy_computation().await },
            Duration::from_millis(2000),
            |state, num| {
                debug!("Worker | update num: {:?}", num);
                Counter { num, ..state }
            },
        );
    });
    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::success(100_000_000) == state.num)
        .for_each(|state| async move {
            info!("  Main | show state: {:?} ", state);
        })
        .await;

    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("B. Async execution will time out");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.async_execute_with_timeout(
            async { heavy_computation().await },
            Duration::from_millis(10),
            |state, num| {
                debug!("Worker | update num: {:?}", num);
                Counter { num, ..state }
            },
        );
    });

    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| state.num.is_fail_with_timeout())
        .for_each(|state| async move {
            info!("  Main | show state: {:?} ", state);
        })
        .await;

    info!("==========================================");
    info!("  Main | Finish");
}

async fn heavy_computation() -> u64 {
    let mut i: u64 = 0;
    for _ in 0..100_000_000 {
        if i % 50_000_000 == 0 {
            tokio::task::yield_now().await;
        }
        i += 1;
    }
    i
}
