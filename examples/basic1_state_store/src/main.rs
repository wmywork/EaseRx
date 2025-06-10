use crate::tracing_setup::tracing_init;
use easerx::AsyncError;
use easerx::{State, StateStore};
use futures_signals::signal::SignalExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

mod tracing_setup;

#[derive(Debug, Clone, Default)]
struct Counter {
    count: i32,
}

impl State for Counter {}

impl Counter {
    fn add_count(self, value: i32) -> Self {
        Self {
            count: self.count + value,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init();

    //Create store
    let store = Arc::new(StateStore::new(Counter::default()));

    info!("==========================================");
    warn!("The main thread and the worker thread execute in parallel, ");
    warn!("the output of the main thread is interleaved with the output of the worker thread.");
    warn!(
        "but the functions of the main thread and the worker thread are still executed in their respective orders"
    );

    info!("  Main thread | A");

    store.with_state(|state| {
        debug!("Worker thread | with_state:{:?}", state);
    })?;

    info!("  Main thread | B");

    store.set_state(|state| {
        debug!("Worker thread | set_state: count + 1");
        state.add_count(1)
    })?;

    info!("  Main thread | C");

    store.with_state(|state| {
        debug!("Worker thread | with_state:{:?}", state);
    })?;

    sleep(Duration::from_millis(10)).await;

    info!("==========================================");
    let current_state = store.await_state().await.unwrap();
    info!("  Main thread | current_state is :{:?}", current_state);

    info!("==========================================");
    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        warn!("states are lossy, they might skip changes");
        for _ in 0..=5 {
            store_clone.set_state(|state| state.add_count(10))?;
        }

        for _ in 0..=5 {
            sleep(Duration::from_millis(10)).await;
            store_clone.set_state(|state| state.add_count(10))?;
        }
        Ok::<(), AsyncError>(())
    });

    store
        .to_signal()
        .stop_if(|state| state.count > 100)
        .for_each(|count| {
            info!("  Main thread | count is :{:?}", count);
            async {}
        })
        .await;

    info!("  Main thread | Finish");
    Ok(())
}
