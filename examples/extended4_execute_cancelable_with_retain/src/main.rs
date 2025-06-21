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

impl Counter {
    pub fn set_num(mut self, value: Async<u64>) -> Self {
        self.num = value;
        self
    }
}

#[tokio::main]
async fn main() {
    tracing_init();

    info!("Using state that implements Clone for value retention");
    info!("==========================================");
    warn!("A. Execution will complete successfully");

    let store = Arc::new(StateStore::new(Counter::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute_cancellable_with_retain(
            CancellationToken::new(),
            |_| 1,
            |state| &state.num,
            |state, num| {
                debug!("Worker | update num: {:?}", num);
                Counter { num, ..state }
            },
        )
    });
    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::success(1) == state.num)
        .for_each(|state| async move {
            info!("  Main | show state: {:?} ", state);
        })
        .await;

    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("B. Cancel from main thread and retain previous value");

    let cancellation_token = CancellationToken::new();
    let control_token = cancellation_token.clone();

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute_cancellable_with_retain(
            cancellation_token,
            |token| heavy_computation_cancellable(token),
            |state| &state.num,
            |state, num| {
                debug!("Worker | update num: {:?}", num);
                Counter { num, ..state }
            },
        );
    });
    control_token.cancel();

    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::fail_with_cancelled(Some(1)) == state.num)
        .for_each(|state| async move {
            info!("  Main | show state: {:?} ", state);
        })
        .await;

    info!("==========================================");
    warn!("C. Cancel from computation Closure and retain previous value");

    store._set_state(|state| state.set_num(Async::success(2)));
    sleep(Duration::from_millis(1)).await;

    let cancellation_token = CancellationToken::new();

    let store_clone = store.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        store_clone.execute_cancellable_with_retain(
            cancellation_token,
            |token| {
                token.cancel();
                heavy_computation_cancellable(token)
            },
            |state| &state.num,
            |state, num| {
                debug!("Worker | update num: {:?}", num);
                Counter { num, ..state }
            },
        );
    });

    let state_flow = store.to_signal();
    state_flow
        .stop_if(|state| Async::fail_with_cancelled(Some(2)) == state.num)
        .for_each(|state| async move {
            info!("  Main | show state: {:?} ", state);
        })
        .await;

    info!("==========================================");
    info!("  Main | Finish");
}

fn heavy_computation_cancellable(cancellation_token: CancellationToken) -> Result<u64, String> {
    let mut i: u64 = 0;
    for _ in 0..200_000_000 {
        // 定期检查是否被取消
        if i % 10_000_000 == 0 && cancellation_token.is_cancelled() {
            debug!("check Cancelled is true and return");
            return Err("Computation was cancelled".to_string());
        }
        i += 1;
    }
    Ok(i)
}
