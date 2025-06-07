use crate::Async;

pub trait ExecutionResult<T: Clone> {
    fn into_async(self) -> Async<T>;
}

impl<T: Clone> ExecutionResult<T> for T {
    fn into_async(self) -> Async<T> {
        Async::success(self)
    }
}

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

impl<T: Clone> ExecutionResult<T> for Option<T> {
    fn into_async(self) -> Async<T> {
        match self {
            Some(value) => Async::success(value),
            None => Async::fail_with_none(None),
        }
    }
}