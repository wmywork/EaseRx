use crate::{Async, AsyncError, State, StateStore};
use futures::stream::StreamExt;
use futures_signals::signal::SignalExt;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

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

    fn set_async_data(self, async_data: Async<String>) -> Self {
        TestState {
            data: async_data,
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
    match state {
        Ok(state) => assert_eq!(state.counter, 10),
        Err(e) => panic!("Failed to get state: {:?}", e),
    }
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

// Test execute functionality
#[tokio::test]
async fn test_execute() {
    let store = StateStore::new(TestState::default());

    // Execute a computation
    store.execute(
        || "Hello, World!".to_string(),
        |state, async_data| state.set_async_data(async_data),
    );

    let mut state_vec = Vec::new();

    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(
        state_vec[2],
        Async::Success {
            value: "Hello, World!".to_string()
        }
    );
}

// Test execute with error
#[tokio::test]
async fn test_execute_with_error() {
    let store = StateStore::new(TestState::default());

    // Execute a computation that returns an error
    store.execute(
        || -> Result<String, &'static str> { Err("Operation failed") },
        |state, async_data| state.set_async_data(async_data),
    );

    let mut state_vec = Vec::new();

    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(
        state_vec[2],
        Async::Fail {
            error: AsyncError::Error("Operation failed".to_string()),
            value: None,
        }
    );
}
// Test execute with Option::none
#[tokio::test]
async fn test_execute_with_none() {
    let store = StateStore::new(TestState::default());

    // Execute a computation that returns an error
    store.execute(
        || None,
        |state, async_data| state.set_async_data(async_data),
    );

    let mut state_vec = Vec::new();
    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(state_vec[2], Async::fail_with_none(None));
}

// Test execute with retain value
#[tokio::test]
async fn test_execute_with_retain() {
    let initial_state = TestState {
        counter: 0,
        data: Async::success("initial".to_string()),
    };

    let store = StateStore::new(initial_state);

    // Execute a computation that fails but should retain previous value
    store.execute_with_retain(
        || -> Result<String, &'static str> { Err("Operation failed") },
        |state| &state.data,
        |state, async_data| state.set_async_data(async_data),
    );

    let state_vec = Arc::new(RwLock::new(Vec::new()));

    store
        .to_signal()
        .stop_if(|_| {
            let len = state_vec.read().unwrap().len();
            len >= 2
        })
        .for_each(|state| {
            state_vec.write().unwrap().push(state.data);
            async {}
        })
        .await;

    let state_vec = state_vec
        .read()
        .unwrap()
        .iter()
        .map(|x| x.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        state_vec[0],
        Async::Success {
            value: "initial".to_string()
        }
    );
    assert_eq!(state_vec[1], Async::Loading(Some("initial".to_string())));
    assert_eq!(
        state_vec[2],
        Async::fail_with_message("Operation failed", Some("initial".to_string()))
    );
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
        |state, async_data| state.set_async_data(async_data),
    );

    // Cancel the operation immediately
    token.cancel();

    let mut state_vec = Vec::new();
    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(state_vec[2], Async::fail_with_cancelled(None));
}

// Test execute_with_timeout
#[tokio::test]
async fn test_execute_with_timeout() {
    let store = StateStore::new(TestState::default());

    // Execute an async computation that takes longer than the timeout
    store.execute_with_timeout(
        || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            "Delayed Result".to_string()
        },
        Duration::from_millis(1),
        |state, async_data| TestState {
            data: async_data,
            ..state
        },
    );

    let mut state_vec = Vec::new();
    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(state_vec[2], Async::fail_with_timeout(None));
}

// Test async_execute
#[tokio::test]
async fn test_async_execute() {
    let store = StateStore::new(TestState::default());

    // Execute an async computation
    store.async_execute(
        async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            "Async Result".to_string()
        },
        |state, async_data| TestState {
            data: async_data,
            ..state
        },
    );

    let mut state_vec = Vec::new();
    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(state_vec[2], Async::success("Async Result".to_string()));
}

// Test async execute with error
#[tokio::test]
async fn test_async_execute_with_error() {
    let store = StateStore::new(TestState::default());

    // Execute a computation that returns an error
    store.async_execute(
        async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            Err("Operation failed")
        },
        |state, async_data| state.set_async_data(async_data),
    );

    let mut state_vec = Vec::new();

    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(
        state_vec[2],
        Async::Fail {
            error: AsyncError::Error("Operation failed".to_string()),
            value: None,
        }
    );
}

// Test async execute with Option::none
#[tokio::test]
async fn test_async_execute_with_none() {
    let store = StateStore::new(TestState::default());

    // Execute a computation that returns an error
    store.async_execute(async { None }, |state, async_data| {
        state.set_async_data(async_data)
    });

    let mut state_vec = Vec::new();
    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(state_vec[2], Async::fail_with_none(None));
}

// Test async execute with retain value
#[tokio::test]
async fn test_async_execute_with_retain() {
    let initial_state = TestState {
        counter: 0,
        data: Async::success("initial".to_string()),
    };

    let store = StateStore::new(initial_state);

    // Execute a computation that fails but should retain previous value
    store.async_execute_with_retain(
        async { Err("Operation failed") },
        |state| &state.data,
        |state, async_data| state.set_async_data(async_data),
    );

    let state_vec = Arc::new(RwLock::new(Vec::new()));

    store
        .to_signal()
        .stop_if(|_| {
            let len = state_vec.read().unwrap().len();
            len >= 2
        })
        .for_each(|state| {
            state_vec.write().unwrap().push(state.data);
            async {}
        })
        .await;

    let state_vec = state_vec
        .read()
        .unwrap()
        .iter()
        .map(|x| x.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        state_vec[0],
        Async::Success {
            value: "initial".to_string()
        }
    );
    assert_eq!(state_vec[1], Async::Loading(Some("initial".to_string())));
    assert_eq!(
        state_vec[2],
        Async::fail_with_message("Operation failed", Some("initial".to_string()))
    );
}

// Test async_execute_cancellable
#[tokio::test]
async fn test_async_execute_cancellable() {
    let store = StateStore::new(TestState::default());
    let token = CancellationToken::new();

    // Execute a cancellable computation
    store.async_execute_cancellable(
        token.clone(),
        |_| async move {
            // Simulate work
            "Result".to_string()
        },
        |state, async_data| state.set_async_data(async_data),
    );

    // Cancel the operation immediately
    token.cancel();

    let mut state_vec = Vec::new();
    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(state_vec[2], Async::fail_with_cancelled(None));
}

// Test async_execute_with_timeout
#[tokio::test]
async fn test_async_execute_with_timeout() {
    let store = StateStore::new(TestState::default());

    // Execute an async computation that takes longer than the timeout
    store.async_execute_with_timeout(
        async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            "Delayed Result".to_string()
        },
        Duration::from_millis(1),
        |state, async_data| TestState {
            data: async_data,
            ..state
        },
    );

    let mut state_vec = Vec::new();
    store
        .to_signal()
        .stop_if(|state| state.data.is_complete())
        .for_each(|state| {
            state_vec.push(state.data);
            async {}
        })
        .await;

    assert_eq!(state_vec[0], Async::Uninitialized);
    assert_eq!(state_vec[1], Async::Loading(None));
    assert_eq!(state_vec[2], Async::fail_with_timeout(None));
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
