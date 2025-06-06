use crate::ExecutionResult;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Async<T: Clone> {
    Uninitialized,
    Loading(Option<T>),
    Success { value: T },
    Fail { error: AsyncError, value: Option<T> },
}

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum AsyncError {
    #[error("{0}")]
    Error(String),
    #[error("Operation returned None!")]
    None,
    #[error("Task was cancelled!")]
    Cancelled,
    #[error("deadline has elapsed!")]
    Timeout,
}

impl AsyncError {
    pub fn is_none(&self) -> bool {
        matches!(self, AsyncError::None)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, AsyncError::Error { .. })
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, AsyncError::Cancelled)
    }
}

impl<T: Clone> Async<T> {
    pub fn complete(&self) -> bool {
        matches!(self, Async::Success { .. } | Async::Fail { .. })
    }

    pub fn should_load(&self) -> bool {
        matches!(self, Async::Uninitialized | Async::Fail { .. })
    }

    pub fn is_incomplete(&self) -> bool {
        matches!(self, Async::Uninitialized | Async::Loading(_))
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Async::Loading { .. })
    }

    pub fn value_ref(&self) -> Option<&T> {
        match self {
            Async::Loading(Some(value)) => Some(&value),
            Async::Success { value } => Some(&value),
            Async::Fail {
                value: Some(value), ..
            } => Some(&value),
            _ => None,
        }
    }

    pub fn success_or_fail_with_retain<'a, F>(self, async_state_getter: F) -> Self
    where
        T: 'a,
        F: FnOnce() -> &'a Async<T>,
    {
        match self {
            Async::Success { value } => Async::success(value),
            Async::Fail { error, .. } => {
                let retained_value = Option::from(async_state_getter());
                Async::fail(error, retained_value)
            }
            other => other,
        }
    }

    pub fn value(self) -> Option<T> {
        match self {
            Async::Uninitialized => None,
            Async::Loading(value) => value,
            Async::Success { value, .. } => Some(value),
            Async::Fail { value, .. } => value,
        }
    }

    pub fn is_fail_with_error(&self) -> bool {
        if let Async::Fail { error, .. } = self {
            error.is_error()
        } else {
            false
        }
    }

    pub fn is_fail_with_none(&self) -> bool {
        if let Async::Fail { error, .. } = self {
            error.is_none()
        } else {
            false
        }
    }

    pub fn is_fail_with_canceled(&self) -> bool {
        if let Async::Fail { error, .. } = self {
            error.is_cancelled()
        } else {
            false
        }
    }

    pub fn loading(value: Option<T>) -> Self {
        Async::Loading(value)
    }

    pub fn success(value: T) -> Self {
        Async::Success { value }
    }

    pub fn fail(error: AsyncError, value: Option<T>) -> Self {
        Async::Fail { error, value }
    }
    pub fn fail_with_cancelled(value: Option<T>) -> Self {
        Async::Fail {
            error: AsyncError::Cancelled,
            value,
        }
    }

    pub fn fail_with_timeout(value: Option<T>) -> Self {
        Async::Fail {
            error: AsyncError::Timeout,
            value,
        }
    }

    pub fn fail_with_message(message: impl Into<String>, value: Option<T>) -> Self {
        let error = AsyncError::Error(message.into());
        Async::Fail { error, value }
    }
}

impl<T: Clone> Default for Async<T> {
    fn default() -> Self {
        Async::Uninitialized
    }
}

impl<T: Clone, R, E> From<Result<R, E>> for Async<T>
where
    R: ExecutionResult<T>,
    E: Into<E> + ToString,
{
    fn from(value: Result<R, E>) -> Self {
        match value {
            Ok(r) => r.into_async(),
            Err(e) => Async::Fail {
                error: AsyncError::Error(e.to_string()),
                value: None,
            },
        }
    }
}

impl<T: Clone> From<&Async<T>> for Option<T> {
    fn from(value: &Async<T>) -> Self {
        match value {
            Async::Loading(Some(value)) => Some(value.clone()),
            Async::Success { value } => Some(value.clone()),
            Async::Fail {
                value: Some(value), ..
            } => Some(value.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninitialized() {
        let uninitialized: Async<i32> = Async::default();
        assert!(!uninitialized.complete());
        assert!(uninitialized.should_load());
        assert!(uninitialized.is_incomplete());
        assert!(!uninitialized.is_loading());
        assert!(uninitialized.value_ref().is_none());
        assert!(uninitialized.value().is_none());
    }
    #[test]
    fn test_loading() {
        let loading = Async::loading(Some(7));
        assert!(!loading.complete());
        assert!(!loading.should_load());
        assert!(loading.is_incomplete());
        assert!(loading.is_loading());
        assert!(loading.value_ref().is_some());
        assert_eq!(loading.value_ref(), Some(7).as_ref());
        assert_eq!(loading.value(), Some(7));

        let loading = Async::loading(None::<i32>);
        assert!(loading.value_ref().is_none());
        assert_eq!(loading.value_ref(), None);
        assert_eq!(loading.value(), None);
    }

    #[test]
    fn test_success() {
        let success = Async::success(8);
        assert!(success.complete());
        assert!(!success.should_load());
        assert!(!success.is_incomplete());
        assert!(!success.is_loading());
        assert!(success.value_ref().is_some());
        assert_eq!(success.value_ref(), Some(&8));
        assert_eq!(success.value(), Some(8));
    }

    #[test]
    fn test_fail() {
        let fail = Async::fail(AsyncError::Error("Connection failed".to_string()), Some(50));
        assert!(fail.complete());
        assert!(fail.should_load());
        assert!(!fail.is_incomplete());
        assert!(!fail.is_loading());
        assert!(fail.value_ref().is_some());
        assert_eq!(fail.value_ref(), Some(&50));
        assert_eq!(fail.value(), Some(50));

        let fail = Async::fail(AsyncError::None, None::<i32>);
        assert!(fail.value_ref().is_none());
        assert_eq!(fail.value_ref(), None);
        assert_eq!(fail.value(), None);

        let fail = Async::fail(AsyncError::Error("Connection failed".to_string()), Some(50));
        assert!(fail.is_fail_with_error());

        let fail = Async::fail(AsyncError::Cancelled, None::<i32>);
        assert!(fail.is_fail_with_canceled());

        let fail = Async::fail(AsyncError::None, None::<i32>);
        assert!(fail.is_fail_with_none());
    }
}
