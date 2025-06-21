use crate::tracing_setup::tracing_init;
use easerx::{Async, State, StateStore};
use futures_signals::signal::SignalExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
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
        store_clone.async_execute_cancellable(
            CancellationToken::new(),
            |_| async { async_heavy_computation().await },
            move |state, num| {
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
    warn!("B. Cancellation from main thread");

    let store = Arc::new(StateStore::new(Counter::default()));
    let cancellation_token = CancellationToken::new();
    let control_token = cancellation_token.clone();

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.async_execute_cancellable(
            cancellation_token,
            |token| async { async_heavy_computation_cancellable(token).await },
            |state, num| {
                debug!("Worker | update num: {:?}", num);
                Counter { num, ..state }
            },
        );
    });
    control_token.cancel();

    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| state.num.is_fail_with_canceled())
        .for_each(|state| async move {
            info!("  Main | show state: {:?} ", state);
        })
        .await;

    info!("==========================================");
    warn!("C. Cancellation from computation closure");

    let store = Arc::new(StateStore::new(Counter::default()));
    let cancellation_token = CancellationToken::new();

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.async_execute_cancellable(
            cancellation_token,
            |token| async {
                token.cancel();
                async_heavy_computation_cancellable(token).await
            },
            |state, num| {
                debug!("Worker | update num: {:?}", num);
                Counter { num, ..state }
            },
        );
    });

    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| state.num.is_fail_with_canceled())
        .for_each(|state| async move {
            info!("  Main | show state: {:?} ", state);
        })
        .await;

    info!("  Main | Finish");
}

async fn async_heavy_computation() -> u64 {
    let mut i: u64 = 0;
    for _ in 0..100_000_000 {
        if i % 50_000_000 == 0 {
            tokio::task::yield_now().await;
        }
        i += 1;
    }
    i
}

async fn async_heavy_computation_cancellable(
    cancellation_token: CancellationToken,
) -> Result<u64, String> {
    let mut i: u64 = 0;
    for _ in 0..100_000_000 {
        // 定期检查是否被取消并让出控制权
        if i % 1_000_000 == 0 {
            if cancellation_token.is_cancelled() {
                debug!("check Cancelled is true and return");
                return Err("Computation was cancelled".to_string());
            }
            tokio::task::yield_now().await;
        }
        i += 1;
    }
    Ok(i)
}
