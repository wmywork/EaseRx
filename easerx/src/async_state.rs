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
    pub fn is_timeout(&self) -> bool {
        matches!(self, AsyncError::Timeout)
    }
}

impl<T: Clone> Async<T> {
    pub fn is_complete(&self) -> bool {
        matches!(self, Async::Success { .. } | Async::Fail { .. })
    }

    pub fn should_load(&self) -> bool {
        matches!(self, Async::Uninitialized | Async::Fail { .. })
    }

    pub fn is_incomplete(&self) -> bool {
        matches!(self, Async::Uninitialized | Async::Loading(_))
    }

    pub fn is_uninitialized(&self) -> bool {
        matches!(self, Async::Uninitialized)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Async::Loading { .. })
    }
    pub fn is_success(&self) -> bool {
        matches!(self, Async::Success { .. })
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, Async::Fail { .. })
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

    pub fn is_fail_with_timeout(&self) -> bool {
        if let Async::Fail { error, .. } = self {
            error.is_timeout()
        } else {
            false
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

    pub fn value_ref_clone(self: &Async<T>) -> Option<T> {
        match self {
            Async::Loading(Some(value)) => Some(value.clone()),
            Async::Success { value } => Some(value.clone()),
            Async::Fail {
                value: Some(value), ..
            } => Some(value.clone()),
            _ => None,
        }
    }

    pub fn cancelled_with_retain(&self) -> Self
    {
        let retained_value: Option<T> = self.value_ref_clone();
        Async::fail_with_cancelled(retained_value)
    }
    
    pub fn success_or_fail_with_retain<'a, F>(self, async_state_getter: F) -> Self
    where
        T: 'a,
        F: FnOnce() -> &'a Async<T>,
    {
        match self {
            Async::Success { value } => Async::success(value),
            Async::Fail { error, .. } => {
                let result = async_state_getter();
                let retained_value: Option<T> = result.value_ref_clone();
                Async::fail(error, retained_value)
            }
            other => other,
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
    pub fn fail_with_none(value: Option<T>) -> Self {
        Async::Fail {
            error: AsyncError::None,
            value,
        }
    }
}

impl<T: Clone> Default for Async<T> {
    fn default() -> Self {
        Async::Uninitialized
    }
}
