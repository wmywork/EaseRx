use thiserror::Error;

/// Represents errors that can occur during asynchronous operations.
///
/// This enum provides a standardized way to represent different types of errors
/// that might occur during asynchronous operations, such as general errors,
/// None values, cancellations, and timeouts.
#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum AsyncError {
    /// A general error with a message describing what went wrong.
    #[error("{0}")]
    Error(String),
    
    /// An operation returned None when a value was expected.
    #[error("Operation returned None!")]
    None,
    
    /// The operation was cancelled before completion.
    #[error("Task was cancelled!")]
    Cancelled,
    
    /// The operation timed out.
    #[error("deadline has elapsed!")]
    Timeout,
}

impl AsyncError {
    /// Returns true if this error represents a None result.
    pub fn is_none(&self) -> bool {
        matches!(self, AsyncError::None)
    }

    /// Returns true if this error is a general error with a message.
    pub fn is_error(&self) -> bool {
        matches!(self, AsyncError::Error { .. })
    }

    /// Returns true if this error represents a cancelled operation.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, AsyncError::Cancelled)
    }
    
    /// Returns true if this error represents a timeout.
    pub fn is_timeout(&self) -> bool {
        matches!(self, AsyncError::Timeout)
    }
}