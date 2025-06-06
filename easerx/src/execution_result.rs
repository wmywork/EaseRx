use crate::{Async, AsyncError};

pub trait ExecutionResult<T:Clone> {
    fn into_async(self) -> Async<T>;
}

impl<T:Clone> ExecutionResult<T> for T {
    fn into_async(self) -> Async<T> {
        Async::Success { value: self }
    }
}

impl<T:Clone, E: ToString> ExecutionResult<T> for Result<T, E> {
    fn into_async(self) -> Async<T> {
        match self {
            Ok(value) => Async::Success { value },
            Err(error) => Async::Fail {
                error: AsyncError::Error(error.to_string()),
                value: None,
            },
        }
    }
}

impl<T:Clone> ExecutionResult<T> for Option<T> {
    fn into_async(self) -> Async<T> {
        match self {
            Some(value) => Async::Success { value },
            None => Async::Fail {
                error: AsyncError::None,
                value: None,
            },
        }
    }
}
