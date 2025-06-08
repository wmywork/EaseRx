use crate::Async;

/// A trait for converting various result types into the `Async<T>` representation.
///
/// This trait provides a unified way to convert different result types (direct values, 
/// `Result<T, E>`, and `Option<T>`) into the `Async<T>` type used throughout the EaseRx framework.
/// It simplifies error handling by automatically converting various error types into
/// the appropriate `Async` variant.
///
/// Implementors of this trait can be used directly with the execution methods of `StateStore`.
pub trait ExecutionResult<T: Clone> {
    /// Converts the implementor into an `Async<T>` representation.
    ///
    /// This method handles the conversion logic for each specific result type,
    /// ensuring appropriate error handling and state representation.
    fn into_async(self) -> Async<T>;
}

/// Implementation for direct values of type `T`.
///
/// This implementation wraps the value in `Async::Success`.
impl<T: Clone> ExecutionResult<T> for T {
    fn into_async(self) -> Async<T> {
        Async::success(self)
    }
}

/// Implementation for `Result<T, E>` where `E` can be converted to a string.
///
/// This implementation converts:
/// - `Ok(value)` to `Async::Success { value }`
/// - `Err(error)` to `Async::Fail` with the error message
impl<T: Clone, E> ExecutionResult<T> for Result<T, E>
where
    E: ToString,
{
    fn into_async(self) -> Async<T> {
        match self {
            Ok(value) => Async::success(value),
            Err(error) => Async::fail_with_message(error.to_string(), None),
        }
    }
}

/// Implementation for `Option<T>`.
///
/// This implementation converts:
/// - `Some(value)` to `Async::Success { value }`
/// - `None` to `Async::Fail` with a None error
impl<T: Clone> ExecutionResult<T> for Option<T> {
    fn into_async(self) -> Async<T> {
        match self {
            Some(value) => Async::success(value),
            None => Async::fail_with_none(None),
        }
    }
}