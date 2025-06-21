use crate::unit_tests::TestState;
use crate::{Async, StateStore};
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time::sleep;
use crate::async_error::AsyncError;

// Test state store initialization
#[tokio::test]
async fn test_state_store_initialization() {
    let initial_state = TestState {
        count: 10,
        data: Async::success("initial".to_string()),
    };

    let store = StateStore::new(initial_state.clone());
    let state = store.get_state();

    assert_eq!(state.count, 10);
    assert!(matches!(
        state.data,
        Async::Success { value } if value == "initial"
    ));
}

// Test synchronous state updates
#[tokio::test]
async fn test_set_state() -> Result<(), AsyncError> {
    let store = StateStore::new(TestState::default());

    // Update state synchronously
    store.set_state(|state| state.add_count(10))?;

    let state = store.await_state().await;
    assert_eq!(
        state,
        Ok(TestState {
            count: 10,
            data: Async::Uninitialized
        })
    );
    Ok(())
}

/*#[tokio::test]
async fn test_set_state_panic() -> Result<(), AsyncError> {
    let store = StateStore::new(TestState::default());

    // Update state synchronously
    store.set_state(|_state| {
        panic!("set_state panic");
        _state.add_count(10)
    })?;

    let state = store.await_state().await;
    assert_eq!(state, Err(AsyncError::error("channel closed")));
    Ok(())
}*/

// Test with_state functionality
#[tokio::test]
async fn test_with_state() -> Result<(), AsyncError> {
    let store = StateStore::new(TestState::default());
    store.set_state(|state| state.add_count(100))?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    store.with_state(move |state| {
        let _ = tx.send(state.count);
    })?;

    let counter = rx.await.unwrap();
    assert_eq!(counter, 100);
    Ok(())
}

#[tokio::test]
async fn test_with_state_panic() -> Result<(), AsyncError> {
    let store = StateStore::new(TestState::default());
    store.set_state(|state| state.add_count(100))?;

    let (tx, rx) = tokio::sync::oneshot::channel();
    store.with_state(move |state| {
        let _ = tx.send(state.count);
        panic!("with_state panic");
    })?;

    let counter = rx.await.unwrap();
    assert_eq!(counter, 100);
    Ok(())
}

#[tokio::test]
async fn test_get_state() -> Result<(), AsyncError> {
    let store = StateStore::new(TestState::default());

    // Update state
    store.set_state(|state| TestState { count: 18, ..state })?;
    sleep(Duration::from_millis(100)).await;
    // Await state and verify
    let state = store.get_state();
    assert_eq!(state.count, 18);
    Ok(())
}

// Test await_state functionality
#[tokio::test]
async fn test_await_state() -> Result<(), AsyncError> {
    let store = StateStore::new(TestState::default());

    // Update state
    store.set_state(|state| TestState { count: 38, ..state })?;

    // Await state and verify
    let state = store.await_state().await.unwrap();
    assert_eq!(state.count, 38);
    Ok(())
}

// Test state stream
#[tokio::test]
async fn test_state_stream() -> Result<(), AsyncError> {
    let store = StateStore::new(TestState::default());
    let mut updates = Vec::new();

    // Create a stream to collect state updates
    let mut stream = store.to_stream();
    let handle = tokio::spawn(async move {
        while let Some(state) = stream.next().await {
            updates.push(state.count);
            if updates.len() >= 3 {
                break;
            }
        }
        updates
    });

    // Make several state updates
    store.set_state(|state| state.set_count(1))?;

    sleep(Duration::from_millis(10)).await;

    store.set_state(|state| state.set_count(2))?;

    sleep(Duration::from_millis(10)).await;

    store.set_state(|state| state.set_count(3))?;

    // Wait for the stream to collect all updates
    let collected_updates = handle.await.unwrap();

    // We should have at least the initial state and our 3 updates
    assert_eq!(collected_updates.len(), 3);
    assert_eq!(collected_updates, [1, 2, 3]);
    Ok(())
}
