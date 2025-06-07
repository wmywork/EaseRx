use crate::Async;
use crate::State;
use crate::{ExecutionResult, execution_result_to_async};
use futures_signals::signal::{Mutable, MutableSignalCloned, SignalExt, SignalStream};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::error::RecvError;
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

    pub fn set_state<F>(&self, reducer: F)
    where
        F: FnOnce(S) -> S + Send + 'static,
    {
        let _ = self.set_state_tx.send(Box::new(reducer));
    }

    pub fn with_state<F>(&self, action: F)
    where
        F: FnOnce(S) + Send + 'static,
    {
        let _ = self.with_state_tx.send(Box::new(action));
    }

    pub fn get_state(&self) -> S {
        self.state.get_cloned()
    }

    pub async fn await_state(&self) -> Result<S, RecvError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.with_state_tx.send(Box::new(|state| {
            let _ = tx.send(state);
        }));
        rx.await
    }

    fn update_async_state<T>(
        set_state_tx: &UnboundedSender<Box<dyn FnOnce(S) -> S + Send>>,
        state_updater: impl FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        async_state: Async<T>,
    ) where
        T: Send + Clone + 'static,
    {
        let _ = set_state_tx.send(Box::new(move |old_state| {
            state_updater(old_state, async_state)
        }));
    }

    fn execute_async_core<T, R, F, U, G>(
        &self,
        computation: F,
        state_updater: U,
        state_getter: Option<G>,
        cancellation_token: Option<CancellationToken>,
    ) where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: Future<Output = R> + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
    {
        let set_state_tx = self.set_state_tx.clone();
        let set_state_tx_retained = self.set_state_tx.clone();
        tokio::spawn(async move {
            // Update the state to indicate loading
            if let Some(getter) = state_getter.clone() {
                let state_updater_clone = state_updater.clone();
                let _ = set_state_tx_retained.send(Box::new(move |old_state| {
                    let previous_result = getter(&old_state);
                    let retained_value = previous_result.value_ref_clone();
                    state_updater_clone(old_state, Async::Loading(retained_value))
                }));
            } else {
                Self::update_async_state(
                    &set_state_tx,
                    state_updater.clone(),
                    Async::Loading(None),
                );
            }
            // Yield to allow the state to be updated before running the computation
            tokio::task::yield_now().await;
            // Run the computation in an async context
            let async_result = if let Some(token) = cancellation_token {
                let token_clone = token.clone();
                tokio::select! {
                    biased;
                    _ = token_clone.cancelled() => Async::fail_with_cancelled(None),
                    result = computation => execution_result_to_async(result),
                }
            } else {
                execution_result_to_async(computation.await)
            };

            let _ = set_state_tx.send(Box::new(move |old_state| {
                let final_result = if let Some(getter) = state_getter {
                    async_result.success_or_fail_with_retain(|| getter(&old_state))
                } else {
                    async_result
                };
                state_updater(old_state, final_result)
            }));
        });
    }

    fn execute_blocking_core<T, R, F, U, G>(
        &self,
        computation: F,
        state_updater: U,
        state_getter: Option<G>,
        cancellation_token: Option<CancellationToken>,
    ) where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce(Option<CancellationToken>) -> R + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
    {
        let set_state_tx = self.set_state_tx.clone();
        let set_state_tx_retained = self.set_state_tx.clone();
        tokio::spawn(async move {
            // Update the state to indicate loading
            if let Some(getter) = state_getter.clone() {
                let state_updater_clone = state_updater.clone();
                let _ = set_state_tx_retained.send(Box::new(move |old_state| {
                    let previous_result = getter(&old_state);
                    let retained_value = previous_result.value_ref_clone();
                    state_updater_clone(old_state, Async::Loading(retained_value))
                }));
            } else {
                Self::update_async_state(
                    &set_state_tx,
                    state_updater.clone(),
                    Async::Loading(None),
                );
            }
            // Yield to allow the state to be updated before running the computation
            tokio::task::yield_now().await;
            // Run the computation in a blocking context
            let async_result = if let Some(token) = cancellation_token {
                let token_clone = token.clone();
                tokio::select! {
                                biased;
                                _ = token_clone.cancelled() => Async::fail_with_cancelled(None),
                                result = tokio::task::spawn_blocking({
                                    let token = token.clone();
                                    move || computation(Some(token))
                                }) => match result {
                    Ok(result) => execution_result_to_async(result),
                    Err(e) => Async::fail_with_message(e.to_string(), None),
                },
                            }
            } else {
                match tokio::task::spawn_blocking(move || computation(None)).await {
                    Ok(r) => execution_result_to_async(r),
                    Err(e) => Async::fail_with_message(e.to_string(), None),
                }
            };

            let _ = set_state_tx.send(Box::new(move |old_state| {
                let final_result = if let Some(getter) = state_getter {
                    async_result.success_or_fail_with_retain(|| getter(&old_state))
                } else {
                    async_result
                };
                state_updater(old_state, final_result)
            }));
        });
    }

    pub fn execute<T, R, F, U>(&self, computation: F, state_updater: U)
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
        );
    }

    pub fn execute_with_retain<T, R, F, G, U>(
        &self,
        computation: F,
        state_getter: G,
        state_updater: U,
    ) where
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
        );
    }

    pub fn execute_cancellable<T, R, F, U>(
        &self,
        cancellation_token: CancellationToken,
        computation: F,
        state_updater: U,
    ) where
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
        );
    }

    pub fn execute_cancellable_with_retain<T, R, F, U, G>(
        &self,
        cancellation_token: CancellationToken,
        computation: F,
        state_getter: G,
        state_updater: U,
    ) where
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
        );
    }

    pub fn async_execute<T, R, F, U>(&self, computation: F, state_updater: U)
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
        );
    }

    pub fn async_execute_with_retain<T, R, F, G, U>(
        &self,
        computation: F,
        state_getter: G,
        state_updater: U,
    ) where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: Future<Output = R> + Send + 'static,
        G: FnOnce(&S) -> &Async<T> + Clone + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        self.execute_async_core(computation, state_updater, Some(state_getter), None);
    }

    pub fn async_execute_cancellable<T, R, F, U, Fut>(
        &self,
        cancellation_token: CancellationToken,
        computation: F,
        state_updater: U,
    ) where
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
        );
    }

    pub fn async_execute_cancellable_with_retain<T, R, F, U, Fut, G>(
        &self,
        cancellation_token: CancellationToken,
        computation: F,
        state_getter: G,
        state_updater: U,
    ) where
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
        );
    }

    pub fn async_execute_with_timeout<T, R, F, U>(
        &self,
        computation: F,
        timeout: std::time::Duration,
        state_updater: U,
    ) where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: Future<Output = R> + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        let set_state_tx = self.set_state_tx.clone();
        tokio::spawn(async move {
            // Update the state to indicate loading
            Self::update_async_state(&set_state_tx, state_updater.clone(), Async::Loading(None));
            // Yield to allow the state to be updated before running the computation
            tokio::task::yield_now().await;
            // Run the computation with a timeout
            let result = tokio::time::timeout(timeout, computation).await;
            let async_result = match result {
                Ok(result) => execution_result_to_async(result),
                Err(_) => Async::fail_with_timeout(None),
            };
            Self::update_async_state(&set_state_tx, state_updater, async_result);
        });
    }

    pub fn execute_with_timeout<T, R, F, U>(
        &self,
        computation: F,
        timeout: std::time::Duration,
        state_updater: U,
    ) where
        T: Clone + Send + 'static,
        R: ExecutionResult<T> + Send + 'static,
        F: FnOnce() -> R + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
    {
        let set_state_tx = self.set_state_tx.clone();
        tokio::spawn(async move {
            // Update the state to indicate loading
            Self::update_async_state(&set_state_tx, state_updater.clone(), Async::Loading(None));
            // Yield to allow the state to be updated before running the computation
            tokio::task::yield_now().await;
            // Run the computation in a blocking context
            let inner_computation = tokio::task::spawn_blocking(computation);
            let async_result = match tokio::time::timeout(timeout, inner_computation).await {
                Ok(result) => match result {
                    Ok(inner_result) => execution_result_to_async(inner_result),
                    Err(inner_error) => Async::fail_with_message(inner_error.to_string(), None),
                },
                Err(_) => Async::fail_with_timeout(None),
            };

            Self::update_async_state(&set_state_tx, state_updater, async_result);
        });
    }
}
