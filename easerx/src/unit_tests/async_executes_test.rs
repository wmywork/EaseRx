use crate::unit_tests::TestState;
use crate::{Async, AsyncError, StateStore};
use futures_signals::signal::SignalExt;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

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

// Test async execute with retain value success
#[tokio::test]
async fn test_async_execute_with_retain_success() {
    let initial_state = TestState {
        data: Async::success("initial".to_string()),
    };

    let store = StateStore::new(initial_state);

    // Execute a computation that fails but should retain previous value
    store.async_execute_with_retain(
        async { Ok::<String, &str>("Operation success".to_string()) },
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

    assert_eq!(state_vec[0], Async::success("initial".to_string()));
    assert_eq!(state_vec[1], Async::Loading(Some("initial".to_string())));
    assert_eq!(
        state_vec[2],
        Async::success("Operation success".to_string())
    );
}

// Test async execute with retain value fail
#[tokio::test]
async fn test_async_execute_with_retain_fail() {
    let initial_state = TestState {
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

    assert_eq!(state_vec[0], Async::success("initial".to_string()));
    assert_eq!(state_vec[1], Async::Loading(Some("initial".to_string())));
    assert_eq!(
        state_vec[2],
        Async::fail_with_message("Operation failed", Some("initial".to_string()))
    );
}

// Test async_execute_cancellable_success
#[tokio::test]
async fn test_async_execute_cancellable_success() {
    let store = StateStore::new(TestState::default());
    let token = CancellationToken::new();

    // Execute a cancellable computation
    store.async_execute_cancellable(
        token,
        |_| async move {
            // Simulate work
            "Result".to_string()
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
    assert_eq!(state_vec[2], Async::success("Result".to_string()));
}

// Test async_execute_cancellable_cancel_inner
#[tokio::test]
async fn test_async_execute_cancellable_cancel_inner() {
    let store = StateStore::new(TestState::default());
    let token = CancellationToken::new();

    // Execute a cancellable computation
    store.async_execute_cancellable(
        token.clone(),
        |token| async move {
            token.cancel();
            "Result".to_string()
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
    assert_eq!(state_vec[2], Async::fail_with_cancelled(None));
}

// Test async_execute_cancellable_cancel_outer
#[tokio::test]
async fn test_async_execute_cancellable_cancel_outer() {
    let store = StateStore::new(TestState::default());
    let token = CancellationToken::new();

    // Execute a cancellable computation
    store.async_execute_cancellable(
        token.clone(),
        |_| async move {
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

// Test async_execute_cancellable_with_retain_success
#[tokio::test]
async fn test_async_execute_cancellable_with_retain_success() {
    let initial_state = TestState {
        data: Async::success("initial".to_string()),
    };
    let store = StateStore::new(initial_state);
    let token = CancellationToken::new();

    // Execute a cancellable computation
    store.async_execute_cancellable_with_retain(
        token.clone(),
        |_| async move { "success".to_string() },
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

    assert_eq!(state_vec[0], Async::success("initial".to_string()));
    assert_eq!(state_vec[1], Async::loading(Some("initial".to_string())));
    assert_eq!(state_vec[2], Async::success("success".to_string()));
}

// Test async_execute_cancellable_with_retain_fail
#[tokio::test]
async fn test_async_execute_cancellable_with_retain_fail() {
    let initial_state = TestState {
        data: Async::success("initial".to_string()),
    };
    let store = StateStore::new(initial_state);
    let token = CancellationToken::new();

    // Execute a cancellable computation
    store.async_execute_cancellable_with_retain(
        token.clone(),
        |_| async move { Err("Result".to_string()) },
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

    assert_eq!(state_vec[0], Async::success("initial".to_string()));
    assert_eq!(state_vec[1], Async::loading(Some("initial".to_string())));
    assert_eq!(
        state_vec[2],
        Async::fail_with_message("Result".to_string(), Some("initial".to_string()))
    );
}

// Test async_execute_cancellable_with_retain_cancel
#[tokio::test]
async fn test_async_execute_cancellable_with_retain_cancel() {
    let initial_state = TestState {
        data: Async::success("initial".to_string()),
    };
    let store = StateStore::new(initial_state);
    let token = CancellationToken::new();

    // Execute a cancellable computation
    store.async_execute_cancellable_with_retain(
        token.clone(),
        |token| async move {
            token.cancel();
            "Result".to_string()
        },
        |state| &state.data,
        |state, async_data| state.set_async_data(async_data),
    );

    // Cancel the operation immediately
    //token.cancel();

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

    assert_eq!(state_vec[0], Async::success("initial".to_string()));
    assert_eq!(state_vec[1], Async::loading(Some("initial".to_string())));
    assert_eq!(
        state_vec[2],
        Async::fail_with_cancelled(Some("initial".to_string()))
    );
}

// Test async_execute_with_timeout_success
#[tokio::test]
async fn test_async_execute_with_timeout_success() {
    let store = StateStore::new(TestState::default());

    // Execute an async computation that takes longer than the timeout
    store.async_execute_with_timeout(
        async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "Delayed Result".to_string()
        },
        Duration::from_millis(50),
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
    assert_eq!(state_vec[2], Async::success("Delayed Result".to_string()));
}

// Test async_execute_with_timeout_fail
#[tokio::test]
async fn test_async_execute_with_timeout_fail() {
    let store = StateStore::new(TestState::default());

    // Execute an async computation that takes longer than the timeout
    store.async_execute_with_timeout(
        async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Err("fail".to_string())
        },
        Duration::from_millis(50),
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
        Async::fail_with_message("fail".to_string(), None)
    );
}

// Test async_execute_with_timeout
#[tokio::test]
async fn test_async_execute_with_timeout() {
    let store = StateStore::new(TestState::default());

    // Execute an async computation that takes longer than the timeout
    store.async_execute_with_timeout(
        async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            "Delayed Result".to_string()
        },
        Duration::from_millis(10),
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
    assert_eq!(state_vec[2], Async::fail_with_timeout(None));
}
