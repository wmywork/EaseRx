use crate::tracing_setup::tracing_init;
use easerx::{State, StateStore};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

mod tracing_setup;

#[derive(Debug, Clone, Default)]
struct Counter {
    _count: i32,
}

impl State for Counter {}

//Create STORE
static STORE: once_cell::sync::Lazy<StateStore<Counter>> =
    once_cell::sync::Lazy::new(|| StateStore::new(Counter::default()));

fn set_state<F>(reducer: F)
where
    F: FnOnce(Counter) -> Counter + Send + 'static,
{
    STORE._set_state(reducer);
}

fn with_state<F>(action: F)
where
    F: FnOnce(Counter) + Send + 'static,
{
    STORE._with_state(action);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init();
    // https://docs.rs/tokio/latest/tokio/macro.select.html
    // state_store use select! to selection branch
    // select! poll the futures in the order
    // loop {
    //     tokio::select! {
    //             biased;
    //             Some(reducer) = set_state_rx.recv() => {...}
    //             Some(action) = with_state_rx.recv() => {...}
    //         }
    // }
    info!("==========================================");
    warn!("Order is : [A, B, W1, W2, S1, S2, S3]");
    info!("  Main thread | A");
    with_state(|_w1| {
        debug!("Worker thread | W1");
        with_state(|_w2| {
            debug!("Worker thread | W2");
            set_state(|s1| {
                set_state(|s2| {
                    set_state(|s3| {
                        debug!("Worker thread | S3");
                        s3
                    });
                    debug!("Worker thread | S2");
                    s2
                });
                debug!("Worker thread | S1");
                s1
            });
        });
    });
    info!("  Main thread | B");
    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("Order is : [A, B, W, S1, W1]");
    info!("  Main thread | A");
    with_state(|_w| {
        debug!("Worker thread | W");
        with_state(|_w1| {
            debug!("Worker thread | W1");
        });
        set_state(|s1| {
            debug!("Worker thread | S1");
            s1
        });
    });
    info!("  Main thread | B");
    sleep(Duration::from_millis(100)).await;

    info!("==========================================");
    warn!("Order is : [A, B, W, S1, S2, W1, W2]");
    info!("  Main thread | A");
    with_state(|_w| {
        debug!("Worker thread | W");
        with_state(|_w1| {
            debug!("Worker thread | W1");
            with_state(|_w2| {
                debug!("Worker thread | W2");
            });
        });
        set_state(|s1| {
            set_state(|s2| {
                debug!("Worker thread | S2");
                s2
            });
            debug!("Worker thread | S1");
            s1
        });
    });
    info!("  Main thread | B");
    sleep(Duration::from_millis(100)).await;
    info!("  Main thread | Finish");
    Ok(())
}
