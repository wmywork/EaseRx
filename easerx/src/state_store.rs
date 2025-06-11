use std::future::Future;
use crate::ExecutionResult;
use crate::State;
use crate::Async;
use futures_signals::signal::{Mutable, MutableSignalCloned, SignalExt, SignalStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use crate::async_error::AsyncError;

/// A reactive state container that manages state updates and provides mechanisms for both synchronous and asynchronous operations.
///
/// `StateStore` is the core component of the EaseRx framework, responsible for managing application state
/// and providing a reactive interface for state changes. It supports various execution modes including
/// synchronous, asynchronous, cancellable, and operations with timeout.
///
/// The state is updated through a message-passing architecture to ensure thread safety and proper
/// sequencing of state updates.
#[derive(Debug, Clone)]
pub struct StateStore<S: State> {
    state: Mutable<S>,
    set_state_tx: UnboundedSender<Box<dyn FnOnce(S) -> S + Send>>,
    with_state_tx: UnboundedSender<Box<dyn FnOnce(S) + Send>>,
}

impl<S: State> StateStore<S> {
    /// Creates a new `StateStore` with the provided initial state.
    ///
    /// This initializes the internal state management system and spawns a background task
    /// to process state updates.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use easerx::{StateStore, State};
    ///
    /// #[derive(Clone)]
    /// struct AppState {
    ///     counter: i32,
    /// }
    ///
    /// impl State for AppState {}
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(AppState { counter: 0 });
    ///     Ok(())
    /// }
    /// ```
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

    /// Converts the state store into a stream of state changes.
    ///
    /// This method returns a `SignalStream` that emits a new value whenever the state changes.
    /// It's useful for reactive UI frameworks or other systems that need to respond to state changes.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use futures_signals::signal_vec::SignalVecExt;
    /// use futures::StreamExt;
    /// use easerx::{EaseRxStreamExt, State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: i32,
    /// }
    /// impl State for TestState {}
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num:0});
    ///     let stream = store.to_stream();
    ///     tokio::spawn(async move {
    ///         stream
    ///             .stop_if(|state|{state.num>1})
    ///             .for_each(|state| {
    ///             println!("State updated: {:?}", state);
    ///             futures::future::ready(())
    ///         }).await;
    ///     });
    ///    Ok(())
    /// }
    /// ```
    pub fn to_stream(&self) -> SignalStream<MutableSignalCloned<S>> {
        self.state.signal_cloned().to_stream()
    }

    /// Returns a signal that represents the current state and its future changes.
    ///
    /// This method returns a `MutableSignalCloned` that can be used to observe state changes
    /// in a reactive manner.
    pub fn to_signal(&self) -> MutableSignalCloned<S> {
        self.state.signal_cloned()
    }

    /// Updates the state by applying a reducer function.
    ///
    /// The reducer function takes the current state and returns a new state.
    /// This operation is performed asynchronously in the background.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use easerx::{State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: i32,
    /// }
    /// impl State for TestState {}
    /// impl TestState{
    ///     fn set_num(self, num: i32) -> Self {
    ///       Self { num, ..self }
    ///     }
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num:0});
    ///     store.set_state(|state| {
    ///         state.set_num(10)
    ///     })?;
    ///    Ok(())
    /// }
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an `AsyncError` if the state update channel is closed.
    pub fn set_state<F>(&self, reducer: F) -> Result<(), AsyncError>
    where
        F: FnOnce(S) -> S + Send + 'static,
    {
        self.set_state_tx
            .send(Box::new(reducer))
            .map_err(|e| AsyncError::error(e.to_string()))
    }

    /// Updates the state by applying a reducer function.
    ///
    /// This method functions the same as set_state() but ignores the return value.
    pub fn _set_state<F>(&self, reducer: F)
    where
        F: FnOnce(S) -> S + Send + 'static,
    {
        let _ = self.set_state_tx
            .send(Box::new(reducer));
    }

    /// Performs an action with the current state without modifying it.
    ///
    /// This is useful for side effects that need to read the current state
    /// but don't need to update it.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use easerx::{State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: i32,
    /// }
    /// impl State for TestState {}
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num:0});
    ///     store.with_state(|state| {
    ///         println!("Current counter value: {}", state.num);
    ///     })?;
    ///    Ok(())
    /// }
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an `AsyncError` if the state action channel is closed.
    pub fn with_state<F>(&self, action: F) -> Result<(), AsyncError>
    where
        F: FnOnce(S) + Send + 'static,
    {
        self.with_state_tx
            .send(Box::new(action))
            .map_err(|e| AsyncError::error(e.to_string()))
    }

    /// Performs an action with the current state without modifying it.
    ///
    /// This method functions the same as with_state() but ignores the return value.
    pub fn _with_state<F>(&self, action: F)
    where
        F: FnOnce(S) + Send + 'static,
    {
        let _ = self.with_state_tx
            .send(Box::new(action));
    }

    /// Returns a clone of the current state.
    ///
    /// This method provides immediate access to the current state value.
    /// Note that the state might change immediately after this call.
    pub fn get_state(&self) -> S {
        self.state.get_cloned()
    }

    /// Returns a future that resolves to the current state.
    ///
    /// This method is useful when you need to ensure you're working with the most
    /// up-to-date state, especially after scheduling state updates.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use easerx::{State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: i32,
    /// }
    /// impl State for TestState {}
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num:0});
    ///     let state = store.await_state().await;
    ///     println!("Current state: {:?}", state);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an `AsyncError` if the state channel is closed or if the oneshot channel fails.
    pub async fn await_state(&self) -> Result<S, AsyncError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let send_result = self.with_state_tx.send(Box::new(|state| {
            let _ = tx.send(state);
        }));
        if let Err(e) = send_result {
            Err(AsyncError::error(e.to_string()))
        } else {
            rx.await.map_err(|e| AsyncError::error(e.to_string()))
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
            .map_err(|e| AsyncError::error(e.to_string()))
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
                state_updater(old_state, Async::loading(retained_value))
            }))
            .map_err(|e| AsyncError::error(e.to_string()))
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
            .map_err(|e| AsyncError::error(e.to_string()))
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
                        Async::loading(None),
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
                        Async::loading(None),
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

    /// Executes a synchronous computation and updates the state with its result.
    ///
    /// This method runs the computation in a blocking task to avoid blocking the async runtime,
    /// and updates the state with the result using the provided state updater function.
    /// The state is first set to `Async::Loading(None)` before executing the computation.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use easerx::{Async, State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: Async<i32>,
    /// }
    /// impl State for TestState {}
    /// impl TestState{
    ///     fn set_num(self, num: Async<i32>) -> Self {
    ///       Self { num, ..self }
    ///     }
    /// }
    /// fn computation() -> Option<i32> {
    ///     Some(888)
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num: Async::default()});
    ///     store.execute(
    ///         || computation(),
    ///         |state, result| {
    ///             state.set_num(result)
    ///         }
    ///     );
    ///   Ok(())
    /// }
    /// ```
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

    /// Executes a synchronous computation and updates the state with its result, retaining previous values.
    ///
    /// Similar to `execute`, but this method retains the previous value when transitioning to the loading state.
    /// This is useful for UI scenarios where you want to show previous data while loading new data.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use easerx::{Async, State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: Async<i32>,
    /// }
    /// impl State for TestState {}
    /// impl TestState{
    ///     fn set_num(self, num: Async<i32>) -> Self {
    ///       Self { num, ..self }
    ///     }
    /// }
    /// fn computation() -> Option<i32> {
    ///     Some(888)
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num: Async::default()});
    ///     store.execute_with_retain(
    ///         || computation(),
    ///         |state| &state.num,
    ///         |state, result| {
    ///             state.set_num(result)
    ///         }
    ///     );
    ///   Ok(())
    /// }
    /// ```
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

    /// Executes a cancellable synchronous computation and updates the state with its result.
    ///
    /// This method allows the computation to be cancelled using the provided cancellation token.
    /// If cancelled, the state will be updated with `Async::Fail` with a cancellation error.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use tokio_util::sync::CancellationToken;
    /// use easerx::{Async, State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: Async<i32>,
    /// }
    /// impl State for TestState {}
    /// impl TestState{
    ///     fn set_num(self, num: Async<i32>) -> Self {
    ///       Self { num, ..self }
    ///     }
    /// }
    /// fn computation(token:CancellationToken) -> Option<i32> {
    ///     for i in 0..1000 {
    ///         if token.is_cancelled() {
    ///             return None;
    ///         }
    ///     }
    ///    Some(888)
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num: Async::default()});
    ///     let token = CancellationToken::new();
    ///     let handle = store.execute_cancellable(
    ///         token.clone(),
    ///         |token| {
    ///             // Check token.is_cancelled() periodically if the operation is long-running
    ///             computation(token)
    ///         },
    ///         |state, result| {
    ///             state.set_num(result)
    ///         }
    ///     );
    ///
    ///     // To cancel the operation:
    ///     token.cancel();
    ///     Ok(())
    /// }
    /// ```
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

    /// Executes a cancellable synchronous computation and updates the state with its result, retaining previous values.
    ///
    /// Combines the functionality of `execute_with_retain` and `execute_cancellable` to provide
    /// a cancellable operation that retains previous values during loading state.
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
                        Async::loading(None),
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
                        Async::loading(None),
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

    /// Executes an asynchronous computation and updates the state with its result.
    ///
    /// This method runs the provided future and updates the state with the result
    /// using the provided state updater function. The state is first set to `Async::Loading(None)`
    /// before executing the computation.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use easerx::{Async, State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: Async<i32>,
    /// }
    /// impl State for TestState {}
    /// impl TestState{
    ///     fn set_num(self, num: Async<i32>) -> Self {
    ///       Self { num, ..self }
    ///     }
    /// }
    /// async fn computation() -> Option<i32> {
    ///     Some(888)
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num: Async::default()});
    ///     store.async_execute(
    ///         async {
    ///             // Fetch data from a database or API
    ///             computation().await
    ///         },
    ///         |state, result| {
    ///             state.set_num(result)
    ///         }
    ///     );
    ///   Ok(())
    /// }
    /// ```
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

    /// Executes an asynchronous computation and updates the state with its result, retaining previous values.
    ///
    /// Similar to `async_execute`, but this method retains the previous value when transitioning
    /// to the loading state. This is useful for UI scenarios where you want to show previous data
    /// while loading new data.
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

    /// Executes a cancellable asynchronous computation and updates the state with its result.
    ///
    /// This method allows the async computation to be cancelled using the provided cancellation token.
    /// If cancelled, the state will be updated with `Async::Fail` with a cancellation error.
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

    /// Executes a cancellable asynchronous computation and updates the state with its result, retaining previous values.
    ///
    /// Combines the functionality of `async_execute_with_retain` and `async_execute_cancellable` to provide
    /// a cancellable operation that retains previous values during loading state.
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

    /// Executes an asynchronous computation with a timeout and updates the state with its result.
    ///
    /// This method runs the provided future with a timeout, and if the timeout is reached,
    /// the state will be updated with `Async::Fail` with a timeout error.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use easerx::{Async, State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: Async<i32>,
    /// }
    /// impl State for TestState {}
    /// impl TestState{
    ///     fn set_num(self, num: Async<i32>) -> Self {
    ///       Self { num, ..self }
    ///     }
    /// }
    /// fn computation() -> Option<i32> {
    ///     Some(888)
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num: Async::default()});
    ///     store.async_execute_with_timeout(
    ///         async {
    ///             // Some potentially slow operation
    ///             tokio::time::sleep(Duration::from_millis(100)).await;
    ///             computation()
    ///         },
    ///         Duration::from_secs(1), // 1 second timeout
    ///         |state, result| {
    ///             state.set_num(result)
    ///         }
    ///     );
    ///   Ok(())
    /// }
    /// ```
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
            Self::update_async_state(&set_state_tx, state_updater.clone(), Async::loading(None))?;
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

    /// Executes a synchronous computation with a timeout and updates the state with its result.
    ///
    /// This method runs the provided computation in a blocking task with a timeout,
    /// and if the timeout is reached, the state will be updated with `Async::Fail` with a timeout error.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use easerx::{Async, State, StateStore};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TestState {
    ///    num: Async<i32>,
    /// }
    /// impl State for TestState {}
    /// impl TestState{
    ///     fn set_num(self, num: Async<i32>) -> Self {
    ///       Self { num, ..self }
    ///     }
    /// }
    /// fn computation() -> Option<i32> {
    ///     Some(888)
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let store = StateStore::new(TestState{num: Async::default()});
    ///     store.execute_with_timeout(
    ///         || {
    ///            // Some potentially slow operation
    ///             std::thread::sleep(Duration::from_millis(100));
    ///             computation()
    ///         },
    ///         Duration::from_secs(1), // 1 second timeout
    ///         |state, result| {
    ///             state.set_num(result)
    ///         }
    ///     );
    ///   Ok(())
    /// }
    /// ```
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
            Self::update_async_state(&set_state_tx, state_updater.clone(), Async::loading(None))?;
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
