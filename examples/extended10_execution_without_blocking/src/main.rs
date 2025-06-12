use crate::tracing_setup::tracing_init;
use easerx::{Async, State, StateStore};
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

async fn normal_function(i: i32) {
    debug!("Worker | start normal_function:{}", i);
    sleep(Duration::from_millis(100)).await;
    debug!("Worker | finish normal_function:{}", i);
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_init();

    info!("tokio: single-threaded runtime");
    info!("==========================================");
    warn!("A. Execution will not blocking normal_function");
    let mut handles = Vec::new();

    for i in 0..2 {
        let handle = tokio::spawn(async move {
            normal_function(i).await;
        });
        handles.push(handle);
    }

    let store = Arc::new(StateStore::new(Counter::default()));
    let store_clone = store.clone();
    tokio::spawn(async move {
        store_clone.execute(
            || heavy_computation(),
            |state, num| {
                debug!("Worker | update num: {:?}", num);
                Counter { num, ..state }
            },
        )
    });

    for handle in handles {
        handle.await.unwrap();
    }

    info!("computation done after normal_function finish");
    loop {
        let state = store.await_state().await;
        if let Ok(Counter { num }) = state {
            if num.is_success() {
                break;
            }
        }
    }

    info!("==========================================");
    warn!("B. Async execution will blocking normal_function");
    let mut handles = Vec::new();

    for i in 0..2 {
        let handle = tokio::spawn(async move {
            normal_function(i).await;
        });
        handles.push(handle);
    }

    let store = Arc::new(StateStore::new(Counter::default()));
    let store_clone = store.clone();
    tokio::spawn(async move {
        store_clone.async_execute(async { heavy_computation() }, |state, num| {
            debug!("Worker | update num: {:?}", num);
            Counter { num, ..state }
        })
    });

    for handle in handles {
        handle.await.unwrap();
    }
    info!("computation done before normal_function finish");

    info!("==========================================");
    info!("  Main | Finish");
}

fn heavy_computation() -> u64 {
    let mut i: u64 = 0;
    for _ in 0..400_000_000 {
        i = i + 1;
    }
    debug!("Main thread | heavy_computation done");
    i
}
