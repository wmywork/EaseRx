use crate::async_error::AsyncError;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents the state of an asynchronous operation with its possible outcomes.
///
/// `Async<T>` is a generic enum that encapsulates the different states an asynchronous
/// operation can be in, including uninitialized, loading, success, and failure states.
/// It provides a uniform way to represent and handle asynchronous state in a reactive application.
///
/// The type parameter `T` represents the successful result type of the operation.
#[derive(Debug, Clone, Eq, PartialEq, Default, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum Async<T: Clone> {
    /// The initial state before any operation has been attempted.
    #[default]
    Uninitialized,

    /// The operation is in progress. May optionally contain the previous value.
    Loading { value: Option<T> },

    /// The operation completed successfully with a result value.
    Success { value: T },

    /// The operation failed. Contains an error and optionally the previous value.
    Fail { error: AsyncError, value: Option<T> },
}

impl<T: Clone> Async<T> {
    /// Returns true if the operation has completed (either successfully or with an error).
    pub fn is_complete(&self) -> bool {
        matches!(self, Async::Success { .. } | Async::Fail { .. })
    }

    /// Returns true if the operation should be (re)loaded.
    ///
    /// This is typically true when the state is either uninitialized or in a failed state.
    pub fn should_load(&self) -> bool {
        matches!(self, Async::Uninitialized | Async::Fail { .. })
    }

    /// Returns true if the operation has not yet completed.
    ///
    /// This is true when the state is either uninitialized or currently loading.
    pub fn is_incomplete(&self) -> bool {
        matches!(self, Async::Uninitialized | Async::Loading { .. })
    }

    /// Returns true if the operation has not been started.
    pub fn is_uninitialized(&self) -> bool {
        matches!(self, Async::Uninitialized)
    }

    /// Returns true if the operation is currently in progress.
    pub fn is_loading(&self) -> bool {
        matches!(self, Async::Loading { .. })
    }

    /// Returns true if the operation completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(self, Async::Success { .. })
    }

    /// Returns true if the operation failed.
    pub fn is_fail(&self) -> bool {
        matches!(self, Async::Fail { .. })
    }

    /// Returns true if the operation failed with a general error.
    pub fn is_fail_with_error(&self) -> bool {
        if let Async::Fail { error, .. } = self {
            error.is_error()
        } else {
            false
        }
    }

    /// Returns true if the operation failed because it returned None.
    pub fn is_fail_with_none(&self) -> bool {
        if let Async::Fail { error, .. } = self {
            error.is_none()
        } else {
            false
        }
    }

    /// Returns true if the operation failed because it was cancelled.
    pub fn is_fail_with_canceled(&self) -> bool {
        if let Async::Fail { error, .. } = self {
            error.is_cancelled()
        } else {
            false
        }
    }

    /// Returns true if the operation failed because it timed out.
    pub fn is_fail_with_timeout(&self) -> bool {
        if let Async::Fail { error, .. } = self {
            error.is_timeout()
        } else {
            false
        }
    }

    /// Consumes the `Async` and returns the contained value if available.
    ///
    /// This method extracts the value from any variant that might contain it:
    /// - `Success` variant returns `Some(value)`
    /// - `Loading` variant with a retained value returns `Some(value)`
    /// - `Fail` variant with a retained value returns `Some(value)`
    /// - Otherwise returns `None`
    pub fn value(self) -> Option<T> {
        match self {
            Async::Uninitialized => None,
            Async::Loading { value } => value,
            Async::Success { value, .. } => Some(value),
            Async::Fail { value, .. } => value,
        }
    }

    /// Returns a reference to the contained value if available.
    ///
    /// Similar to `value()` but returns a reference instead of consuming the `Async`.
    pub fn value_ref(&self) -> Option<&T> {
        match self {
            Async::Loading { value: Some(value) } => Some(value),
            Async::Success { value } => Some(value),
            Async::Fail {
                value: Some(value), ..
            } => Some(value),
            _ => None,
        }
    }

    /// Returns a clone of the contained value if available.
    ///
    /// This method is similar to `value_ref()` but returns a clone of the value
    /// rather than a reference.
    pub fn value_ref_clone(self: &Async<T>) -> Option<T> {
        match self {
            Async::Loading { value: Some(value) } => Some(value.clone()),
            Async::Success { value } => Some(value.clone()),
            Async::Fail {
                value: Some(value), ..
            } => Some(value.clone()),
            _ => None,
        }
    }

    /// Sets or updates the retained value in `Loading` or `Fail` states.
    ///
    /// This method is useful when you want to update the retained value
    /// without changing the state itself.
    pub fn set_retain_value(mut self, value: Option<T>) -> Self {
        match self {
            Async::Loading { .. } => {
                self = Async::loading(value);
            }
            Async::Fail { error, .. } => {
                self = Async::fail(error, value);
            }
            _ => {}
        }
        self
    }

    /// Creates a new `Async` in the `Loading` state.
    ///
    /// Optionally includes a retained value from a previous operation.
    pub fn loading(value: Option<T>) -> Self {
        Async::Loading { value }
    }

    /// Creates a new `Async` in the `Success` state with the provided value.
    pub fn success(value: T) -> Self {
        Async::Success { value }
    }

    /// Creates a new `Async` in the `Fail` state with the provided error and optional retained value.
    pub fn fail(error: AsyncError, value: Option<T>) -> Self {
        Async::Fail { error, value }
    }

    /// Creates a new `Async` in the `Fail` state with a cancellation error.
    pub fn fail_with_cancelled(value: Option<T>) -> Self {
        Async::Fail {
            error: AsyncError::Cancelled,
            value,
        }
    }

    /// Creates a new `Async` in the `Fail` state with a timeout error.
    pub fn fail_with_timeout(value: Option<T>) -> Self {
        Async::Fail {
            error: AsyncError::Timeout,
            value,
        }
    }

    /// Creates a new `Async` in the `Fail` state with a general error message.
    pub fn fail_with_message(message: impl Into<String>, value: Option<T>) -> Self {
        let error = AsyncError::error(message.into());
        Async::Fail { error, value }
    }

    /// Creates a new `Async` in the `Fail` state with a None error.
    pub fn fail_with_none(value: Option<T>) -> Self {
        Async::Fail {
            error: AsyncError::None,
            value,
        }
    }
}
