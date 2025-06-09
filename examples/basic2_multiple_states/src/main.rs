use futures_signals::signal::SignalExt;
use futures_signals::map_ref;
use crate::tracing_setup::tracing_init;
use easerx::{State, StateStore, combine_state_flow, AsyncError};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
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

    info!("  Main thread | Create stores");
    let store1 = Arc::new(StateStore::new(Counter { count: 1 }));
    let store2 = Arc::new(StateStore::new(Counter { count: 2 }));

    info!("==========================================");
    warn!("example: Subscribe to state changes using the for_each() method.");

    let store1_clone = store1.clone();
    let store2_clone = store2.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        store1_clone.set_state(|state| {
            debug!("Worker thread | set_state1: count + 10");
            state.add_count(10)
        })?;
        sleep(Duration::from_millis(50)).await;
        store2_clone.set_state(|state| {
            debug!("Worker thread | set_state2: count + 10");
            state.add_count(10)
        })?;
        Ok::<(), AsyncError>(())
    });

    info!("  Main thread | combine state flow");
    let state_flow = combine_state_flow!{store1.to_signal(), store2.to_signal()};
    state_flow
        .stop_if(|(state1, state2)| {
            state1.count >= 11 && state2.count >= 12
        })
        .for_each(|(state1, state2)| async move {
            info!(
                "  Main thread | state1: {:?} , state2: {:?}",
                state1, state2
            );
        })
        .await;

    info!("==========================================");
    warn!("example: Subscribe to state changes using the next() method.");

    let store1_clone = store1.clone();
    let store2_clone = store2.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        store1_clone.set_state(|state| {
            debug!("Worker thread | set_state1: count + 100");
            state.add_count(100)
        })?;
        sleep(Duration::from_millis(100)).await;
        store2_clone.set_state(|state| {
            debug!("Worker thread | set_state2: count + 100");
            state.add_count(100)
        })?;
        Ok::<(), AsyncError>(())
    });

    let mut state_stream = combine_state_flow!(store1.to_signal(), store2.to_signal()).to_stream();
    loop {
        if let Some((state1, state2)) = state_stream.next().await {
            debug!(
            "Worker thread | state1: {:?} , state2: {:?}",
            state1, state2
        );
            if state1.count >= 111 && state2.count >= 112 {
                break;
            }
        } else {
            break;
        }
    }

    info!("  Main thread | Finish");
    Ok(())
}
