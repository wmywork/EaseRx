use crate::{Async, AsyncError, ExecutionResult};

#[cfg(test)]
mod tests {
    use super::*;

    // Test success_or_fail_with_retain functionality
    #[test]
    fn test_success_or_fail_with_retain() {
        // Test with success state
        let success = Async::success(42);
        let retained = Async::success(100);

        let result = success.success_or_fail_with_retain(|| &retained);
        assert!(matches!(result, Async::Success { value: 42 }));

        // Test with fail state
        let fail = Async::fail(AsyncError::Error("Error".to_string()), None);
        let retained = Async::success(100);

        let result = fail.success_or_fail_with_retain(|| &retained);
        match result {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Error(msg) if msg == "Error"));
                assert!(matches!(value, Some(v) if v == 100));
            }
            _ => panic!("Expected Async::Fail with retained value"),
        }

        // Test with loading state
        let loading = Async::loading(Some(42));
        let retained = Async::success(100);

        let result = loading.success_or_fail_with_retain(|| &retained);
        assert!(matches!(result, Async::Loading(Some(42))));
    }

    // Test from trait implementation for Result
    #[test]
    fn test_from_result() {
        // Test with Result::Ok
        let result: Result<i32, &str> = Ok(42);
        let async_result: Async<i32> = result.into();

        assert!(matches!(async_result, Async::Success { value: 42 }));

        // Test with Result::Err
        let result: Result<i32, &str> = Err("error");
        let async_result: Async<i32> = result.into();

        match async_result {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Error(msg) if msg == "error"));
                assert!(value.is_none());
            }
            _ => panic!("Expected Async::Fail variant"),
        }
    }

    // Test From trait implementation for &Async<T>
    #[test]
    fn test_from_async_ref() {
        // Test with Success
        let success = Async::success(42);
        let option: Option<i32> = (&success).into();
        assert_eq!(option, Some(42));

        // Test with Loading
        let loading = Async::loading(Some(42));
        let option: Option<i32> = (&loading).into();
        assert_eq!(option, Some(42));

        // Test with Loading (None)
        let loading = Async::loading(None::<i32>);
        let option: Option<i32> = (&loading).into();
        assert_eq!(option, None);

        // Test with Fail (with value)
        let fail = Async::fail(AsyncError::Error("error".to_string()), Some(42));
        let option: Option<i32> = (&fail).into();
        assert_eq!(option, Some(42));

        // Test with Fail (without value)
        let fail = Async::fail(AsyncError::Error("error".to_string()), None::<i32>);
        let option: Option<i32> = (&fail).into();
        assert_eq!(option, None);

        // Test with Uninitialized
        let uninitialized = Async::<i32>::Uninitialized;
        let option: Option<i32> = (&uninitialized).into();
        assert_eq!(option, None);
    }

    // Test error state helpers
    #[test]
    fn test_error_state_helpers() {
        // Test is_fail_with_error
        let fail = Async::fail(AsyncError::Error("error".to_string()), None::<i32>);
        assert!(fail.is_fail_with_error());

        let fail = Async::fail(AsyncError::None, None::<i32>);
        assert!(!fail.is_fail_with_error());

        // Test is_fail_with_none
        let fail = Async::fail(AsyncError::None, None::<i32>);
        assert!(fail.is_fail_with_none());

        let fail = Async::fail(AsyncError::Error("error".to_string()), None::<i32>);
        assert!(!fail.is_fail_with_none());

        // Test is_fail_with_canceled
        let fail = Async::fail(AsyncError::Cancelled, None::<i32>);
        assert!(fail.is_fail_with_canceled());

        let fail = Async::fail(AsyncError::Error("error".to_string()), None::<i32>);
        assert!(!fail.is_fail_with_canceled());
    }

    // Test factory methods
    #[test]
    fn test_factory_methods() {
        // Test fail_with_cancelled
        let fail = Async::<i32>::fail_with_cancelled(Some(42));
        match fail {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Cancelled));
                assert_eq!(value, Some(42));
            }
            _ => panic!("Expected Async::Fail with Cancelled error"),
        }

        // Test fail_with_timeout
        let fail = Async::<i32>::fail_with_timeout(Some(42));
        match fail {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Timeout));
                assert_eq!(value, Some(42));
            }
            _ => panic!("Expected Async::Fail with Timeout error"),
        }

        // Test fail_with_message
        let fail = Async::<i32>::fail_with_message("custom error", Some(42));
        match fail {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Error(msg) if msg == "custom error"));
                assert_eq!(value, Some(42));
            }
            _ => panic!("Expected Async::Fail with Error"),
        }
    }

    // Test AsyncError methods
    #[test]
    fn test_async_error_methods() {
        let none_error = AsyncError::None;
        assert!(none_error.is_none());
        assert!(!none_error.is_error());
        assert!(!none_error.is_cancelled());

        let error = AsyncError::Error("message".to_string());
        assert!(!error.is_none());
        assert!(error.is_error());
        assert!(!error.is_cancelled());

        let cancelled = AsyncError::Cancelled;
        assert!(!cancelled.is_none());
        assert!(!cancelled.is_error());
        assert!(cancelled.is_cancelled());
    }

    // Test complex state transitions
    #[test]
    fn test_complex_state_transitions() {
        // Create a complex type for testing
        #[derive(Clone, Debug, PartialEq)]
        struct User {
            id: i32,
            name: String,
        }

        // Start with uninitialized
        let mut state = Async::<User>::Uninitialized;
        assert!(state.is_incomplete());
        assert!(state.should_load());

        // Transition to loading
        let user = User {
            id: 1,
            name: "John".to_string(),
        };
        state = Async::loading(Some(user.clone()));
        assert!(state.is_incomplete());
        assert!(!state.should_load());
        assert!(state.is_loading());

        // Transition to success
        let updated_user = User {
            id: 1,
            name: "John Doe".to_string(),
        };
        state = Async::success(updated_user.clone());
        assert!(state.complete());
        assert!(!state.should_load());
        assert!(!state.is_incomplete());
        assert_eq!(state.value_ref(), Some(&updated_user));

        // Transition to fail with retained value
        state = Async::fail(
            AsyncError::Error("Update failed".to_string()),
            Some(updated_user.clone()),
        );
        assert!(state.complete());
        assert!(state.should_load());
        assert!(!state.is_incomplete());
        assert_eq!(state.value_ref(), Some(&updated_user));

        // Transition back to loading
        state = Async::loading(None);
        assert!(state.is_incomplete());
        assert!(!state.should_load());
        assert!(state.is_loading());
        assert!(state.value_ref().is_none());
    }
}
