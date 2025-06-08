# EaseRx - Reactive MVI Framework for Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

EaseRx is a reactive Model-View-Intent (MVI) framework for Rust, designed to reduce the mental overhead of asynchronous programming while providing an intuitive state management solution.

## Overview

Rust's learning curve can be steep, especially when dealing with asynchronous programming and state management. EaseRx aims to simplify this by providing a structured architecture that allows developers to write asynchronous code in a synchronous style, with automatic error conversion.

## Core Features

- **Reactive State Management**: Complete solution for managing application state with automatic propagation of state changes
- **Simplified Async Programming**: Write asynchronous code in a synchronous style
- **Automatic Error Conversion**: Unified error handling mechanism that automatically converts various error types
- **Cancellable Operations**: Support for cancellation of long-running operations
- **Timeout Handling**: Built-in support for operation timeouts
- **Retained Values**: Keep previous values while loading new data

## Key Components

### StateStore

`StateStore<S>` is the core component of EaseRx, responsible for managing application state and providing a reactive interface for state changes:

```
let store = StateStore::new(AppState { counter: 0 });

// Update state
store.set_state(|state| {
    state.add_count(1)
});

// Execute an operation that updates state
store.execute(
    || computation(),
    | state, result| {
        sate.set_data(result)
    }
);
```

### Async State Representation

`Async<T>` encapsulates the different states of an asynchronous operation:

- `Uninitialized`: Initial state
- `Loading`: Operation in progress (optionally retaining previous value)
- `Success`: Operation completed successfully with a result
- `Fail`: Operation failed with an error (optionally retaining previous value)

```
match state.data {
    Async::Uninitialized => println!("Not started"),
    Async::Loading(_) => println!("Loading..."),
    Async::Success { value } => println!("Data: {}", value),
    Async::Fail { error, .. } => println!("Error: {}", error),
}
```

## Execution Methods

EaseRx provides various execution methods to suit different scenarios:

- **Synchronous Operations**:
  - `execute`: Basic synchronous operation
  - `execute_with_retain`: Retain previous values during loading
  - `execute_cancellable`: Support for cancellation
  - `execute_with_timeout`: Automatic timeout handling

- **Asynchronous Operations**:
  - `async_execute`: Basic asynchronous operation
  - `async_execute_with_retain`: Retain previous values during loading
  - `async_execute_cancellable`: Support for cancellation
  - `async_execute_with_timeout`: Automatic timeout handling

## Use Cases

- **UI State Management**: Manage component states in GUI applications
- **Network Request Handling**: Handle loading states, successful responses, and error scenarios
- **Data Processing Pipelines**: Build data transformation, filtering, and aggregation operations

## Getting Started

Add EaseRx to your `Cargo.toml`:

```toml
[dependencies]
easerx = "0.1.0"
```

Basic usage example:

```rust
use easerx::{StateStore, State, Async};
use std::time::Duration;

#[derive(Clone)]
struct AppState {
    data: Async<String>,
}

impl State for AppState {}

fn main() {
    // Create a new state store
    let store = StateStore::new(AppState { 
        data: Async::Uninitialized 
    });
    
    // Execute an asynchronous operation with timeout
    store.async_execute_with_timeout(
        async {
            // Simulate network request
            tokio::time::sleep(Duration::from_millis(100)).await;
            "Data loaded successfully".to_string()
        },
        Duration::from_secs(1),
        |mut state, result| {
            state.data = result;
            state
        }
    );
    
    // Subscribe to state changes
    let stream = store.to_stream();
    // Process state updates...
}
```

## Technical Dependencies

- [tokio](https://github.com/tokio-rs/tokio): Asynchronous runtime
- [futures-signals](https://github.com/Pauan/rust-signals): Reactive programming support
- [thiserror](https://github.com/dtolnay/thiserror): Error handling
- [pin-project](https://github.com/taiki-e/pin-project): Pin projection

## License

This project is licensed under the MIT License - see the LICENSE file for details. 