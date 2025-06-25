<img src="/_images/logo.png" alt="EaseRx Logo" class="half-width-img">

# Introduction

## What is EaseRx?

EaseRx is a reactive Model-View-Intent (MVI) framework for Rust, designed to reduce the mental overhead of asynchronous programming while providing an intuitive state management solution.

Rust's learning curve can be steep, especially when dealing with asynchronous programming and state management. EaseRx aims to simplify this by providing a structured architecture that allows developers to write asynchronous code in a synchronous style, with automatic error conversion.

## Core Features

- **Reactive State Management**: Complete solution for managing application state with automatic propagation of state changes.
- **Simplified Async Programming**: Write asynchronous code in a synchronous style.
- **Automatic Error Conversion**: Unified error handling mechanism that automatically converts various error types.
- **Cancellable Operations**: Support for cancellation of long-running operations.
- **Timeout Handling**: Built-in support for operation timeouts.
- **Retained Values**: Keep previous values while loading new data.

## Design Principles

1.  **Simplicity**: API design is clear and easy to understand and use.
2.  **Consistency**: Maintains consistent interfaces and behavior patterns.
3.  **Composability**: Components can be flexibly combined to meet different needs.
4.  **Type Safety**: Leverages Rust's type system to ensure code safety.
5.  **Performance**: Prioritizes performance in design decisions.

Whether you are building a Terminal User Interface (TUI), a web application, or any other event-driven software, EaseRx provides the tools you need to manage complexity and build reliable applications.

This book will guide you through the core concepts, from getting started to advanced usage patterns, with practical examples to help you along the way.