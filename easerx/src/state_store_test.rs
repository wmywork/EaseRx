use crate::{Async, AsyncError, ExecutionResult, State, StateStore};
use futures_signals::signal::SignalExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

// Define a simple state for testing
#[derive(Clone, Debug, PartialEq)]
struct TestState {
    counter: i32,
    data: Async<String>,
    list: Async<Vec<i32>>,
}

impl State for TestState {}

impl Default for TestState {
    fn default() -> Self {
        TestState {
            counter: 0,
            data: Async::Uninitialized,
            list: Async::Uninitialized,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_signals::signal::SignalStream;
    use std::time::Instant;
    use tokio::time::sleep;

    // Test state store initialization
    #[tokio::test]
    async fn test_state_store_initialization() {
        let initial_state = TestState {
            counter: 10,
            data: Async::success("initial".to_string()),
            list: Async::default(),
        };

        let store = StateStore::new(initial_state.clone());
        let state = store.get_state();

        assert_eq!(state.counter, 10);
        assert!(matches!(
            state.data,
            Async::Success { value } if value == "initial"
        ));
        assert!(matches!(state.list, Async::Uninitialized));
    }

    // Test synchronous state updates
    #[tokio::test]
    async fn test_set_state() {
        let store = StateStore::new(TestState::default());

        // Update state synchronously
        store.set_state(|state| TestState {
            counter: state.counter + 1,
            ..state
        });

        // Allow time for the async queue to process
        sleep(Duration::from_millis(10)).await;

        let state = store.get_state();
        assert_eq!(state.counter, 1);
    }

    // Test with_state functionality
    #[tokio::test]
    async fn test_with_state() {
        let store = StateStore::new(TestState::default());
        let counter = Arc::new(Mutex::new(0));

        {
            let counter_clone = counter.clone();
            store.with_state(move |state| {
                *counter_clone.lock().unwrap() = state.counter;
            });
        }

        // Allow time for the async queue to process
        sleep(Duration::from_millis(10)).await;

        assert_eq!(*counter.lock().unwrap(), 0);
    }

    // Test await_state functionality
    #[tokio::test]
    async fn test_await_state() {
        let store = StateStore::new(TestState::default());

        // Update state
        store.set_state(|state| TestState {
            counter: 42,
            ..state
        });

        // Await state and verify
        let state = store.await_state().await.unwrap();
        assert_eq!(state.counter, 42);
    }

    // Test execute functionality
    #[tokio::test]
    async fn test_execute() {
        let store = StateStore::new(TestState::default());

        // Execute a computation
        store.execute(
            || "Hello, World!".to_string(),
            |state, async_data| TestState {
                data: async_data,
                ..state
            },
        );

        // Allow time for the async computation to complete
        sleep(Duration::from_millis(50)).await;

        let state = store.get_state();
        assert!(matches!(
            state.data,
            Async::Success { value } if value == "Hello, World!"
        ));
    }

    // Test execute with error
    #[tokio::test]
    async fn test_execute_with_error() {
        let store = StateStore::new(TestState::default());

        // Execute a computation that returns an error
        store.execute(
            || -> Result<String, &'static str> { Err("Operation failed") },
            |state, async_data| TestState {
                data: async_data,
                ..state
            },
        );

        // Allow time for the async computation to complete
        sleep(Duration::from_millis(50)).await;

        let state = store.get_state();
        match state.data {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Error(msg) if msg == "Operation failed"));
                assert!(value.is_none());
            }
            _ => panic!("Expected Async::Fail variant"),
        }
    }

    // Test execute_with_retain
    #[tokio::test]
    async fn test_execute_with_retain() {
        let initial_state = TestState {
            counter: 0,
            data: Async::success("initial".to_string()),
            list: Async::default(),
        };

        let store = StateStore::new(initial_state);

        // Execute a computation that fails but should retain previous value
        store.execute_with_retain(
            || -> Result<String, &'static str> { Err("Operation failed") },
            |state| &state.data,
            |state, async_data| TestState {
                data: async_data,
                ..state
            },
        );

        // Allow time for the async computation to complete
        sleep(Duration::from_millis(50)).await;

        let state = store.get_state();
        match state.data {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Error(msg) if msg == "Operation failed"));
                assert!(matches!(value, Some(v) if v == "initial"));
            }
            _ => panic!("Expected Async::Fail variant with retained value"),
        }
    }

    // Test execute_cancellable
    #[tokio::test]
    async fn test_execute_cancellable() {
        let store = StateStore::new(TestState::default());
        let token = CancellationToken::new();

        // Execute a cancellable computation
        store.execute_cancellable(
            token.clone(),
            |_| {
                // Simulate work
                "Result".to_string()
            },
            |state, async_data| TestState {
                data: async_data,
                ..state
            },
        );

        // Cancel the operation immediately
        token.cancel();

        // Allow time for cancellation to be processed
        sleep(Duration::from_millis(50)).await;

        let state = store.get_state();
        match state.data {
            Async::Fail { error, .. } => {
                assert!(matches!(error, AsyncError::Cancelled));
            }
            _ => panic!("Expected Async::Fail with Cancelled error"),
        }
    }

    // Test async_execute
    #[tokio::test]
    async fn test_async_execute() {
        let store = StateStore::new(TestState::default());

        // Execute an async computation
        store.async_execute(
            async {
                sleep(Duration::from_millis(10)).await;
                "Async Result".to_string()
            },
            |state, async_data| TestState {
                data: async_data,
                ..state
            },
        );

        // Verify loading state
        let loading_state = store.get_state();
        assert!(matches!(loading_state.data, Async::Loading(None)));

        // Allow time for the async computation to complete
        sleep(Duration::from_millis(50)).await;

        let state = store.get_state();
        assert!(matches!(
            state.data,
            Async::Success { value } if value == "Async Result"
        ));
    }

    // Test async_execute_with_timeout
    #[tokio::test]
    async fn test_async_execute_with_timeout() {
        let store = StateStore::new(TestState::default());

        // Execute an async computation that takes longer than the timeout
        store.async_execute_with_timeout(
            async {
                sleep(Duration::from_millis(100)).await;
                "Delayed Result".to_string()
            },
            Duration::from_millis(10),
            |state, async_data| TestState {
                data: async_data,
                ..state
            },
        );

        // Allow time for the timeout to occur
        sleep(Duration::from_millis(50)).await;

        let state = store.get_state();
        match state.data {
            Async::Fail { error, .. } => {
                assert!(matches!(error, AsyncError::Timeout));
            }
            _ => panic!("Expected Async::Fail with Timeout error"),
        }
    }

    // Test state stream
    #[tokio::test]
    async fn test_state_stream() {
        let store = StateStore::new(TestState::default());
        let mut updates = Vec::new();

        // Create a stream to collect state updates
        let stream = store.to_stream();
        let handle = tokio::spawn(async move {
            let mut stream = SignalStream::new(stream);
            while let Some(state) = stream.next().await {
                updates.push(state.counter);
                if updates.len() >= 3 {
                    break;
                }
            }
            updates
        });

        // Make several state updates
        store.set_state(|state| TestState {
            counter: 1,
            ..state
        });

        sleep(Duration::from_millis(10)).await;

        store.set_state(|state| TestState {
            counter: 2,
            ..state
        });

        sleep(Duration::from_millis(10)).await;

        store.set_state(|state| TestState {
            counter: 3,
            ..state
        });

        // Wait for the stream to collect all updates
        let collected_updates = handle.await.unwrap();

        // We should have at least the initial state and our 3 updates
        assert!(collected_updates.len() >= 3);
        assert_eq!(collected_updates[collected_updates.len() - 3..], [1, 2, 3]);
    }
}
