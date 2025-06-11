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
    warn!("example: execute");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute(
            || heavy_computation(),
            |state, num| {
                debug!("Worker thread | update num: {:?}", num);
                Counter { num, ..state }
            },
        )
    });
    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::Success { value: 200_000_000 } == state.num)
        .for_each(|state| async move {
            info!("  Main thread | show state: {:?} ", state);
        })
        .await;

    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("example: execute with Result Ok");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute(
            || heavy_computation_result(false),
            |state, num| {
                debug!("Worker thread | update num: {:?}", num);
                Counter { num, ..state }
            },
        )
    });
    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::Success { value: 200_000_000 } == state.num)
        .for_each(|state| async move {
            info!("  Main thread | show state: {:?} ", state);
        })
        .await;

    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("example: execute with Result Err");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute(
            || heavy_computation_result(true),
            |state, num| {
                debug!("Worker thread | update num: {:?}", num);
                Counter { num, ..state }
            },
        )
    });
    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| {
            Async::fail_with_message("Computation was cancelled".to_string(), None) == state.num
        })
        .for_each(|state| async move {
            info!("  Main thread | show state: {:?} ", state);
        })
        .await;

    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("example: execute with Option Some");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute(
            || heavy_computation_option(false),
            |state, num| {
                debug!("Worker thread | update num: {:?}", num);
                Counter { num, ..state }
            },
        )
    });
    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::success(200_000_000) == state.num)
        .for_each(|state| async move {
            info!("  Main thread | show state: {:?} ", state);
        })
        .await;

    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("example: execute with Option None");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute(
            || heavy_computation_option(true),
            |state, num| {
                debug!("Worker thread | update num: {:?}", num);
                Counter { num, ..state }
            },
        )
    });
    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::fail_with_none(None) == state.num)
        .for_each(|state| async move {
            info!("  Main thread | show state: {:?} ", state);
        })
        .await;

    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    info!("  Main thread | Finish");
}

fn heavy_computation() -> u64 {
    let mut i: u64 = 0;
    for _ in 0..200_000_000 {
        i = i + 1;
    }
    i
}

fn heavy_computation_result(fail: bool) -> Result<u64, String> {
    let mut i: u64 = 0;
    for _ in 0..200_000_000 {
        if i % 10_000_000 == 0 && fail {
            return Err("Computation was cancelled".to_string());
        }
        i = i + 1;
    }
    Ok(i)
}

fn heavy_computation_option(none: bool) -> Option<u64> {
    let mut i: u64 = 0;
    for _ in 0..200_000_000 {
        if i % 10_000_000 == 0 && none {
            return None;
        }
        i = i + 1;
    }
    Some(i)
}
