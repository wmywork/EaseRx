use crate::{Async, AsyncError};

pub trait ExecutionResult<T: Clone> {
    fn into_async(self) -> Async<T>;
}

impl<T: Clone> ExecutionResult<T> for T {
    fn into_async(self) -> Async<T> {
        Async::Success { value: self }
    }
}

impl<T: Clone, E> ExecutionResult<T> for Result<T, E>
where
    E: Into<E> + ToString,
{
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

impl<T: Clone> ExecutionResult<T> for Option<T> {
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

pub fn execution_result_to_async<T: Clone, R>(result: R) -> Async<T>
where
    R: ExecutionResult<T>,
{
    result.into_async()
}