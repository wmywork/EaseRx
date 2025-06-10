use crate::{EaseRxStreamExt, State, StateStore};
use futures::StreamExt;
use std::sync::Arc;
use crate::async_error::AsyncError;

#[derive(Clone, Debug, PartialEq)]
struct TestStreamState {
    pub data: i32,
    pub progress: f64,
}

impl State for TestStreamState {}

impl Default for TestStreamState {
    fn default() -> Self {
        Self {
            data: 0,
            progress: 0.0,
        }
    }
}

impl TestStreamState {
    pub fn set_data(self, data: i32) -> Self {
        Self { data, ..self }
    }

    pub fn set_progress(self, progress: f64) -> Self {
        Self { progress, ..self }
    }
}

#[tokio::test]
async fn test_stream_ext_for_each() -> Result<(), AsyncError> {
    let store = Arc::new(StateStore::new(TestStreamState::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        store_clone.set_state(|state| state.set_data(1))?;
        store_clone.set_state(|state| state.set_progress(0.1))?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        store_clone.set_state(|state| state.set_data(2))?;
        store_clone.set_state(|state| state.set_progress(0.2))?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        store_clone.set_state(|state| state.set_data(3))?;
        store_clone.set_state(|state| state.set_progress(0.3))?;
        Ok::<(), AsyncError>(())
    });

    let mut data_vec = Vec::new();
    let mut progress_vec = Vec::new();
    let state_flow = store.to_stream();
    state_flow
        .stop_if(|state| state.data >= 3)
        .for_each(|state| {
            data_vec.push(state.data);
            progress_vec.push(state.progress);
            async {}
        })
        .await;

    assert_eq!(data_vec, vec![0, 1, 2, 3]);
    assert_eq!(progress_vec, vec![0.0, 0.1, 0.2, 0.3]);
    Ok(())
}

#[tokio::test]
async fn test_stream_ext_loop() -> Result<(), AsyncError> {
    let store = Arc::new(StateStore::new(TestStreamState::default()));

    let store_clone = store.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        store_clone.set_state(|state| state.set_data(1))?;
        store_clone.set_state(|state| state.set_progress(0.1))?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        store_clone.set_state(|state| state.set_data(2))?;
        store_clone.set_state(|state| state.set_progress(0.2))?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        store_clone.set_state(|state| state.set_data(3))?;
        store_clone.set_state(|state| state.set_progress(0.3))?;
        Ok::<(), AsyncError>(())
    });

    let mut data_vec = Vec::new();
    let mut progress_vec = Vec::new();
    let mut state_flow = store.to_stream();
    loop {
        match state_flow.next().await {
            Some(state) => {
                data_vec.push(state.data);
                progress_vec.push(state.progress);
                if state.data >= 3 {
                    break;
                }
            }
            None => break,
        }
    }
    assert_eq!(data_vec, vec![0, 1, 2, 3]);
    assert_eq!(progress_vec, vec![0.0, 0.1, 0.2, 0.3]);
    Ok(())
}
