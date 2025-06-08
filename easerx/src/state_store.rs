use crate::ExecutionResult;
use crate::State;
use crate::{Async, AsyncError};
use futures_signals::signal::{Mutable, MutableSignalCloned, SignalExt, SignalStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct StateStore<S: State> {
    state: Mutable<S>,
    set_state_tx: UnboundedSender<Box<dyn FnOnce(S) -> S + Send>>,
    with_state_tx: UnboundedSender<Box<dyn FnOnce(S) + Send>>,
}

impl<S: State> StateStore<S> {
    pub fn new(initial_state: S) -> Self {
        let state = Mutable::new(initial_state);
        let (set_state_tx, set_state_rx) =
            tokio::sync::mpsc::unbounded_channel::<Box<dyn FnOnce(S) -> S + Send>>();
        let (with_state_tx, with_state_rx) =
            tokio::sync::mpsc::unbounded_channel::<Box<dyn FnOnce(S) + Send>>();

        let state_clone = state.clone();

        tokio::spawn(async move {
            Self::process_queue(state_clone, set_state_rx, with_state_rx).await;
        });

        StateStore {
            state,
            set_state_tx,
            with_state_tx,
        }
    }

    async fn process_queue(
        state: Mutable<S>,
        mut set_state_rx: UnboundedReceiver<Box<dyn FnOnce(S) -> S + Send>>,
        mut with_state_rx: UnboundedReceiver<Box<dyn FnOnce(S) + Send>>,
    ) {
        loop {
            tokio::select! {
                biased;
                Some(reducer) = set_state_rx.recv() => {
                    let new_state = reducer(state.get_cloned());
                    state.set(new_state)
                }
                Some(action) = with_state_rx.recv() => {
                    action(state.get_cloned());
                }
                else => break,
            }
        }
    }

    pub fn to_stream(&self) -> SignalStream<MutableSignalCloned<S>> {
        self.state.signal_cloned().to_stream()
    }

    pub fn to_signal(&self) -> MutableSignalCloned<S> {
        self.state.signal_cloned()
    }

    pub fn set_state<F>(&self, reducer: F) -> Result<(), AsyncError>
    where
        F: FnOnce(S) -> S + Send + 'static,
    {
        self.set_state_tx
            .send(Box::new(reducer))
            .map_err(|e| AsyncError::Error(e.to_string()))
    }

    pub fn with_state<F>(&self, action: F) -> Result<(), AsyncError>
    where
        F: FnOnce(S) + Send + 'static,
    {
        self.with_state_tx
            .send(Box::new(action))
            .map_err(|e| AsyncError::Error(e.to_string()))
    }

    pub fn get_state(&self) -> S {
        self.state.get_cloned()
    }

    pub async fn await_state(&self) -> Result<S, AsyncError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let send_result = self.with_state_tx.send(Box::new(|state| {
            let _ = tx.send(state);
        }));
        if let Err(e) = send_result {
            Err(AsyncError::Error(e.to_string()))
        } else {
            rx.await.map_err(|e| AsyncError::Error(e.to_string()))
        }
    }

    fn update_async_state<T>(
        set_state_tx: &UnboundedSender<Box<dyn FnOnce(S) -> S + Send>>,
        state_updater: impl FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        async_state: Async<T>,
    ) -> Result<(), AsyncError>
    where
        T: Send + Clone + 'static,
    {
        set_state_tx
            .send(Box::new(move |old_state| {
                state_updater(old_state, async_state)
            }))
            .map_err(|e| AsyncError::Error(e.to_string()))
    }

    async fn run_computation_cancelable<T, R, F>(
        computation: F,
        token: CancellationToken,
    ) -> Async<T>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce(Option<CancellationToken>) -> R + Send + 'static,
    {
        tokio::select! {
            biased;
            _ = token.cancelled() => Async::fail_with_cancelled(None),
            result = tokio::task::spawn_blocking({
                let token = token.clone();
                move || computation(Some(token))
            }) => match result {
                Ok(result) => result.into_async(),
                Err(e) => Async::fail_with_message(e.to_string(), None),
            },
        }
    }

    async fn run_computation<T, R, F>(computation: F) -> Async<T>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce(Option<CancellationToken>) -> R + Send + 'static,
    {
        match tokio::task::spawn_blocking(move || computation(None)).await {
            Ok(result) => result.into_async(),
            Err(e) => Async::fail_with_message(e.to_string(), None),
        }
    }

    fn update_async_to_loading_with_retain<T, G>(
        set_state_tx: &UnboundedSender<Box<dyn FnOnce(S) -> S + Send>>,
        state_updater: impl FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        state_getter: G,
    ) -> Result<(), AsyncError>
    where
        T: Send + Clone + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
    {
        set_state_tx
            .send(Box::new(move |old_state| {
                let previous_result = state_getter(&old_state);
                let retained_value = previous_result.value_ref_clone();
                state_updater(old_state, Async::Loading(retained_value))
            }))
            .map_err(|e| AsyncError::Error(e.to_string()))
    }

    fn update_async_cancelable_with_retain<T, G>(
        set_state_tx: &UnboundedSender<Box<dyn FnOnce(S) -> S + Send>>,
        state_updater: impl FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        state_getter: G,
        async_result: Async<T>,
        token_is_cancelled: bool,
    ) -> Result<(), AsyncError>
    where
        T: Send + Clone + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
    {
        set_state_tx
            .send(Box::new(move |old_state| {
                let retained = state_getter(&old_state).value_ref_clone();
                let final_result = if token_is_cancelled {
                    Async::fail_with_cancelled(retained)
                } else {
                    async_result.set_retain_value(retained)
                };
                state_updater(old_state, final_result)
            }))
            .map_err(|e| AsyncError::Error(e.to_string()))
    }

    fn execute_blocking_core<T, R, F, U, G>(
        &self,
        computation: F,
        state_updater: U,
        state_getter: Option<G>,
        cancellation_token: Option<CancellationToken>,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce(Option<CancellationToken>) -> R + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
    {
        let set_state_tx = self.set_state_tx.clone();
        let updater_loading = state_updater.clone();
        tokio::task::spawn(async move {
            match (cancellation_token, state_getter) {
                (Some(token), Some(getter)) => {
                    // If we have a getter and a cancellation token, we can update the state to loading with the retained value
                    let getter_loading = getter.clone();
                    Self::update_async_to_loading_with_retain(&set_state_tx, updater_loading, getter_loading)?;
                    // Yield to allow the state to be updated before running the computation
                    tokio::task::yield_now().await;
                    // Run the computation in a blocking context with cancellation support
                    let async_result =
                        Self::run_computation_cancelable(computation, token.clone()).await;
                    // Send the result back to the state store
                    Self::update_async_cancelable_with_retain(
                        &set_state_tx,
                        state_updater,
                        getter,
                        async_result,
                        token.is_cancelled(),
                    )
                }
                (Some(token), None) => {
                    // If we have a cancellation token but no getter, we can update the state to loading with None
                    Self::update_async_state(
                        &set_state_tx,
                        state_updater.clone(),
                        Async::Loading(None),
                    )?;
                    // Yield to allow the state to be updated before running the computation
                    tokio::task::yield_now().await;
                    // Run the computation in a blocking context with cancellation support
                    let async_result =
                        Self::run_computation_cancelable(computation, token.clone()).await;
                    // Send the result back to the state store
                    let final_result = if token.is_cancelled() {
                        Async::fail_with_cancelled(None)
                    } else {
                        async_result
                    };
                    Self::update_async_state(&set_state_tx, state_updater, final_result)
                }
                (None, Some(getter)) => {
                    // If we have a getter but no cancellation token, we can update the state to loading with the retained value
                    let getter_loading = getter.clone();
                    Self::update_async_to_loading_with_retain(
                        &set_state_tx,
                        updater_loading,
                        getter_loading,
                    )?;
                    // Yield to allow the state to be updated before running the computation
                    tokio::task::yield_now().await;
                    // Run the computation in a blocking context without cancellation support
                    let async_result = Self::run_computation(computation).await;
                    Self::update_async_cancelable_with_retain(
                        &set_state_tx,
                        state_updater,
                        getter,
                        async_result,
                        false,
                    )
                }

                (None, None) => {
                    // If we have neither a getter nor a cancellation token, we can update the state to loading with None
                    Self::update_async_state(
                        &set_state_tx,
                        state_updater.clone(),
                        Async::Loading(None),
                    )?;
                    // Yield to allow the state to be updated before running the computation
                    tokio::task::yield_now().await;
                    // Run the computation in a blocking context without cancellation support
                    let async_result = Self::run_computation(computation).await;
                    // Send the result back to the state store
                    Self::update_async_state(&set_state_tx, state_updater, async_result)
                }
            }
        })
    }

    pub fn execute<T, R, F, U>(
        &self,
        computation: F,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Send + Clone + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce() -> R + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        self.execute_blocking_core(
            move |_| computation(),
            state_updater,
            None::<fn(&S) -> &Async<T>>,
            None,
        )
    }

    pub fn execute_with_retain<T, R, F, G, U>(
        &self,
        computation: F,
        state_getter: G,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce() -> R + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        self.execute_blocking_core(
            move |_| computation(),
            state_updater,
            Some(state_getter),
            None,
        )
    }

    pub fn execute_cancellable<T, R, F, U>(
        &self,
        cancellation_token: CancellationToken,
        computation: F,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce(CancellationToken) -> R + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        self.execute_blocking_core(
            move |token| computation(token.unwrap()),
            state_updater,
            None::<fn(&S) -> &Async<T>>,
            Some(cancellation_token),
        )
    }

    pub fn execute_cancellable_with_retain<T, R, F, U, G>(
        &self,
        cancellation_token: CancellationToken,
        computation: F,
        state_getter: G,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce(CancellationToken) -> R + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
    {
        self.execute_blocking_core(
            move |token| computation(token.unwrap()),
            state_updater,
            Some(state_getter),
            Some(cancellation_token),
        )
    }

    async fn run_async_computation_cancelable<T, R, F>(
        computation: F,
        token: CancellationToken,
    ) -> Async<T>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: Future<Output = R> + Send + 'static,
    {
        tokio::select! {
            biased;
            _ = token.cancelled() => Async::fail_with_cancelled(None),
            result = computation => result.into_async(),
        }
    }

    fn execute_async_core<T, R, F, U, G>(
        &self,
        computation: F,
        state_updater: U,
        state_getter: Option<G>,
        cancellation_token: Option<CancellationToken>,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: Future<Output = R> + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
    {
        let set_state_tx = self.set_state_tx.clone();
        let updater_loading = state_updater.clone();
        tokio::task::spawn(async move {
            match (cancellation_token, state_getter) {
                (Some(token), Some(getter)) => {
                    // If we have a getter and a cancellation token, we can update the state to loading with the retained value
                    let getter_loading = getter.clone();
                    Self::update_async_to_loading_with_retain(&set_state_tx, updater_loading, getter_loading)?;
                    // Yield to allow the state to be updated before running the computation
                    tokio::task::yield_now().await;
                    // Run the computation in a blocking context with cancellation support
                    let async_result =
                        Self::run_async_computation_cancelable(computation, token.clone()).await;
                    // Send the result back to the state store
                    Self::update_async_cancelable_with_retain(
                        &set_state_tx,
                        state_updater,
                        getter,
                        async_result,
                        token.is_cancelled(),
                    )
                }
                (Some(token), None) => {
                    // If we have a cancellation token but no getter, we can update the state to loading with None
                    Self::update_async_state(
                        &set_state_tx,
                        state_updater.clone(),
                        Async::Loading(None),
                    )?;
                    // Yield to allow the state to be updated before running the computation
                    tokio::task::yield_now().await;
                    // Run the computation in a blocking context with cancellation support
                    let async_result =
                        Self::run_async_computation_cancelable(computation, token.clone()).await;
                    // Send the result back to the state store
                    let final_result = if token.is_cancelled() {
                        Async::fail_with_cancelled(None)
                    } else {
                        async_result
                    };
                    Self::update_async_state(&set_state_tx, state_updater, final_result)
                }
                (None, Some(getter)) => {
                    // If we have a getter but no cancellation token, we can update the state to loading with the retained value
                    let getter_loading = getter.clone();
                    Self::update_async_to_loading_with_retain(&set_state_tx, updater_loading, getter_loading)?;
                    // Yield to allow the state to be updated before running the computation
                    tokio::task::yield_now().await;
                    // Run the computation in a blocking context without cancellation support
                    let async_result = computation.await.into_async();
                    // Send the result back to the state store
                    Self::update_async_cancelable_with_retain(
                        &set_state_tx,
                        state_updater,
                        getter,
                        async_result,
                        false,
                    )
                }
                (None, None) => {
                    // If we have neither a getter nor a cancellation token, we can update the state to loading with None
                    Self::update_async_state(
                        &set_state_tx,
                        state_updater.clone(),
                        Async::Loading(None),
                    )?;
                    // Yield to allow the state to be updated before running the computation
                    tokio::task::yield_now().await;
                    // Run the computation in a blocking context without cancellation support
                    let async_result = computation.await.into_async();
                    // Send the result back to the state store
                    Self::update_async_state(&set_state_tx, state_updater, async_result)
                }
            }
        })
    }

    pub fn async_execute<T, R, F, U>(
        &self,
        computation: F,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: Future<Output = R> + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        self.execute_async_core(
            computation,
            state_updater,
            None::<fn(&S) -> &Async<T>>,
            None,
        )
    }

    pub fn async_execute_with_retain<T, R, F, G, U>(
        &self,
        computation: F,
        state_getter: G,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: Future<Output = R> + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        self.execute_async_core(computation, state_updater, Some(state_getter), None)
    }

    pub fn async_execute_cancellable<T, R, F, U, Fut>(
        &self,
        cancellation_token: CancellationToken,
        computation: F,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        self.execute_async_core(
            computation(cancellation_token.clone()),
            state_updater,
            None::<fn(&S) -> &Async<T>>,
            Some(cancellation_token),
        )
    }

    pub fn async_execute_cancellable_with_retain<T, R, F, U, Fut, G>(
        &self,
        cancellation_token: CancellationToken,
        computation: F,
        state_getter: G,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
    {
        self.execute_async_core(
            computation(cancellation_token.clone()),
            state_updater,
            Some(state_getter),
            Some(cancellation_token),
        )
    }

    pub fn async_execute_with_timeout<T, R, F, U>(
        &self,
        computation: F,
        timeout: std::time::Duration,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: Future<Output = R> + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        let set_state_tx = self.set_state_tx.clone();
        tokio::spawn(async move {
            // Update the state to indicate loading
            Self::update_async_state(&set_state_tx, state_updater.clone(), Async::Loading(None))?;
            // Yield to allow the state to be updated before running the computation
            tokio::task::yield_now().await;
            // Run the computation with a timeout
            let result = tokio::time::timeout(timeout, computation).await;
            let async_result = match result {
                Ok(result) => result.into_async(),
                Err(_) => Async::fail_with_timeout(None),
            };
            Self::update_async_state(&set_state_tx, state_updater, async_result)
        })
    }

    pub fn execute_with_timeout<T, R, F, U>(
        &self,
        computation: F,
        timeout: std::time::Duration,
        state_updater: U,
    ) -> JoinHandle<Result<(), AsyncError>>
    where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce() -> R + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        let set_state_tx = self.set_state_tx.clone();
        tokio::spawn(async move {
            // Update the state to indicate loading
            Self::update_async_state(&set_state_tx, state_updater.clone(), Async::Loading(None))?;
            // Yield to allow the state to be updated before running the computation
            tokio::task::yield_now().await;
            // Run the computation in a blocking context
            let inner_computation = tokio::task::spawn_blocking(computation);
            let result = tokio::time::timeout(timeout, inner_computation).await;
            let async_result = match result {
                Ok(inner_result) => match inner_result {
                    Ok(final_result) => final_result.into_async(),
                    Err(final_error) => Async::fail_with_message(final_error.to_string(), None),
                },
                Err(_) => Async::fail_with_timeout(None),
            };

            Self::update_async_state(&set_state_tx, state_updater, async_result)
        })
    }
}
