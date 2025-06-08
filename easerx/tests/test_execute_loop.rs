use std::time::Instant;
use futures_signals::signal::SignalExt;
use easerx::StateStore;
use crate::common::TestState;

mod common;

const LOOP_COUNT: u64 = 1_0;
#[tokio::test]
async fn test_execute_loop() {
    let state = TestState::default();
    let store = StateStore::new(state);
    let tick = Instant::now();
    for i in 0..LOOP_COUNT {
        store.execute_with_retain(move || i, |state| &state.num, |state, num| state.set_num(num));
        store.to_signal()
            .stop_if(|x| {
                x.num.is_complete()
            })
            .for_each(|_| {
                async {}
            })
            .await;
    }
    let elapsed = tick.elapsed();
    println!("  Main thread | elapsed: {:?}", elapsed);
}