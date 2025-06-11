use crate::tracing_setup::tracing_init;
use easerx::{Async, AsyncError, State, StateStore};
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

    info!("State must impl Clone for retain value");
    info!("==========================================");
    warn!("A. execution will be successful ");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute_with_retain(
            || fibonacci_result(1),
            |state| &state.num,
            |state, num| {
                debug!("Worker thread | update num: {:?}", num);
                Counter { num, ..state }
            },
        )
    });

    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::success(1) == state.num)
        .for_each(|state| async move {
            info!("  Main thread | show state: {:?} ", state);
        })
        .await;

    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("B. execute with retain value (will be fail and retain previous value)");

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute_with_retain(
            || fibonacci_result(93),
            |state| &state.num,
            |state, num| {
                debug!("Worker thread | update num: {:?}", num);
                Counter { num, ..state }
            },
        );
    });

    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| {
            Async::fail_with_message("calculation overflow at n=93", Some(1)) == state.num
        })
        .for_each(|state| async move {
            info!("  Main thread | show state: {:?} ", state);
        })
        .await;

    info!("==========================================");
    info!("  Main thread | Finish");
}

fn fibonacci_result(n: u64) -> Result<u64, String> {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        match a.checked_add(b) {
            Some(next) => {
                a = b;
                b = next;
            }
            None => return Err(format!("calculation overflow at n={}", n)),
        }
    }
    Ok(a)
}
