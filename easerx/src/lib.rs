//! # EaseRX
//! EaseRx is a reactive Model-View-Intent (MVI) framework for Rust, 
//! designed to reduce the mental overhead of asynchronous programming while providing an intuitive state management solution.
//!
//! ## Overview
//!
//! Rust's learning curve can be steep, especially when dealing with asynchronous programming and state management. 
//! EaseRx aims to simplify this by providing a structured architecture that allows developers to write 
//! asynchronous code in a synchronous style, with automatic error conversion.
//!
//! ## Core Features
//!
//! - **Reactive State Management**: Complete solution for managing application state with automatic propagation of state changes
//! - **Simplified Async Programming**: Write asynchronous code in a synchronous style
//! - **Automatic Error Conversion**: Unified error handling mechanism that automatically converts various error types
//! - **Cancellable Operations**: Support for cancellation of long-running operations
//! - **Timeout Handling**: Built-in support for operation timeouts
//! - **Retained Values**: Keep previous values while loading new data
//!
//! ## Key Components
//!
//! ### StateStore
//!
//! [`StateStore`] is the core component of EaseRx, responsible for managing application state and providing 
//! a reactive interface for state changes. It supports various execution modes including synchronous, 
//! asynchronous, cancellable, and operations with timeout.
//!
//! ```
//! use futures_signals::signal::SignalExt;
//! use easerx::{StateStore, State, Async};
//! #[derive(Clone, Debug, PartialEq)]
//! struct AppState { num: i32, data: Async<String> }
//! impl State for AppState {}
//!
//! impl AppState{
//!     fn set_num(self, num: i32) -> Self {
//!       Self { num, ..self }
//!     }
//!    fn set_data(self, data: Async<String>) -> Self {
//!      Self { data , ..self}
//!    }
//! }
//! #[tokio::main]
//! async fn main()-> Result<(), Box<dyn std::error::Error>> {
//! let store = StateStore::new(AppState { num: 0, data: Async::Uninitialized });
//!
//! // Update state
//! store.set_state(|state| {
//!     state.set_num(1)
//! })?;
//!
//! // Execute an operation that updates state
//! store.execute(
//!     || "example computation".to_string(),
//!     |state, result| {
//!         state.set_data(result)
//!     }
//! );
//!
//! store.to_signal()
//!     .stop_if(|state| {state.data.is_complete()})
//!     .for_each(|state| {
//!         println!("state: {:?}", state);
//!         async {}
//!     }).await;
//! Ok(())
//!}
//! ```
//!
//! ### Async State Representation
//!
//! [`Async<T>`] encapsulates the different states of an asynchronous operation:
//!
//! - `Uninitialized`: Initial state before any operation has been attempted
//! - `Loading`: Operation in progress (optionally retaining previous value)
//! - `Success`: Operation completed successfully with a result value
//! - `Fail`: Operation failed with an error (optionally retaining previous value)
//!
//! ```
//! # use easerx::Async;
//! # let state_data = Async::<String>::Uninitialized;
//! match state_data {
//!     Async::Uninitialized => println!("Not started"),
//!     Async::Loading { .. } => println!("Loading..."),
//!     Async::Success { value } => println!("Data: {}", value),
//!     Async::Fail { error, .. } => println!("Error: {}", error),
//! }
//! ```
//!
//! ### Execution Result Conversion
//!
//! The [`ExecutionResult`] trait provides a unified way to convert different result types 
//! (direct values, `Result<T, E>`, and `Option<T>`) into the [`Async<T>`] type. This simplifies 
//! error handling by automatically converting various error types into the appropriate `Async` variant.
//!
//! ### Stream Extensions
//!
//! The [`EaseRxStreamExt`] trait extends the functionality of `Stream` types with additional 
//! utility methods like `stop_if`, which creates a stream that stops producing items once a 
//! predicate returns true.
//!
//! ## Execution Methods
//!
//! [`StateStore`] provides various execution methods to suit different scenarios:
//!
//! - **Synchronous Operations**:
//!   - `execute`: Basic synchronous operation
//!   - `execute_with_retain`: Retain previous values during loading
//!   - `execute_cancellable`: Support for cancellation
//!   - `execute_with_timeout`: Automatic timeout handling
//!
//! - **Asynchronous Operations**:
//!   - `async_execute`: Basic asynchronous operation
//!   - `async_execute_with_retain`: Retain previous values during loading
//!   - `async_execute_cancellable`: Support for cancellation
//!   - `async_execute_with_timeout`: Automatic timeout handling
//!
//! ## Design Principles
//!
//! 1. **Simplicity**: API design is clear and easy to understand and use
//! 2. **Consistency**: Maintains consistent interfaces and behavior patterns
//! 3. **Composability**: Components can be flexibly combined to meet different needs
//! 4. **Type Safety**: Leverages Rust's type system to ensure code safety
//! 5. **Performance**: Prioritizes performance in design decisions

mod async_state;
mod async_error;
mod state_store;
mod execution_result;
mod stream_ext;
pub mod macros;

pub use async_state::*;
pub use async_error::*;
pub use state_store::*;
pub use execution_result::*;
pub use stream_ext::*;

/// A trait for types that can be used as state in a [`StateStore`].
///
/// This trait is a marker trait that requires the implementing type to be 
/// `Clone`, `Send`, `Sync`, and `'static`. These constraints ensure that 
/// the state can be safely shared and manipulated across threads in an 
/// asynchronous environment.
pub trait State: Clone + Send + Sync + 'static {}

#[cfg(test)]
mod unit_tests;