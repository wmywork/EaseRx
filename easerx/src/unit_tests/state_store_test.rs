use crate::{Async, State, StateStore};
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time::sleep;

// Define a simple state for testing
#[derive(Clone, Debug, PartialEq)]
struct TestState {
    counter: i32,
    data: Async<String>,
}

impl State for TestState {}

impl Default for TestState {
    fn default() -> Self {
        TestState {
            counter: 0,
            data: Async::Uninitialized,
        }
    }
}

impl TestState {
    fn add_count(self, value: i32) -> Self {
        TestState {
            counter: self.counter + value,
            ..self
        }
    }

    fn set_count(self, value: i32) -> Self {
        TestState {
            counter: value,
            ..self
        }
    }
}

// Test state store initialization
#[tokio::test]
async fn test_state_store_initialization() {
    let initial_state = TestState {
        counter: 10,
        data: Async::success("initial".to_string()),
    };

    let store = StateStore::new(initial_state.clone());
    let state = store.get_state();

    assert_eq!(state.counter, 10);
    assert!(matches!(
        state.data,
        Async::Success { value } if value == "initial"
    ));
}

// Test synchronous state updates
#[tokio::test]
async fn test_set_state() {
    let store = StateStore::new(TestState::default());

    // Update state synchronously
    store.set_state(|state| state.add_count(10));

    let state = store.await_state().await;
    assert_eq!(
        state,
        Ok(TestState {
            counter: 10,
            data: Async::Uninitialized
        })
    );
}

// Test with_state functionality
#[tokio::test]
async fn test_with_state() {
    let store = StateStore::new(TestState::default());
    store.set_state(|state| state.add_count(100));

    let (tx, rx) = tokio::sync::oneshot::channel();
    store.with_state(move |state| {
        let _ = tx.send(state.counter);
    });

    let counter = rx.await.unwrap();
    assert_eq!(counter, 100);
}

#[tokio::test]
async fn test_get_state() {
    let store = StateStore::new(TestState::default());

    // Update state
    store.set_state(|state| TestState {
        counter: 18,
        ..state
    });
    sleep(Duration::from_millis(100)).await;
    // Await state and verify
    let state = store.get_state();
    assert_eq!(state.counter, 18);
}

// Test await_state functionality
#[tokio::test]
async fn test_await_state() {
    let store = StateStore::new(TestState::default());

    // Update state
    store.set_state(|state| TestState {
        counter: 38,
        ..state
    });

    // Await state and verify
    let state = store.await_state().await.unwrap();
    assert_eq!(state.counter, 38);
}

// Test state stream
#[tokio::test]
async fn test_state_stream() {
    let store = StateStore::new(TestState::default());
    let mut updates = Vec::new();

    // Create a stream to collect state updates
    let mut stream = store.to_stream();
    let handle = tokio::spawn(async move {
        while let Some(state) = stream.next().await {
            updates.push(state.counter);
            if updates.len() >= 3 {
                break;
            }
        }
        updates
    });

    // Make several state updates
    store.set_state(|state| state.set_count(1));

    sleep(Duration::from_millis(10)).await;

    store.set_state(|state| state.set_count(2));

    sleep(Duration::from_millis(10)).await;

    store.set_state(|state| state.set_count(3));

    // Wait for the stream to collect all updates
    let collected_updates = handle.await.unwrap();

    // We should have at least the initial state and our 3 updates
    assert_eq!(collected_updates.len(), 3);
    assert_eq!(collected_updates, [1, 2, 3]);
}
