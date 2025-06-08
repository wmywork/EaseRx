use crate::{Async, AsyncError};

#[test]
fn test_uninitialized() {
    let uninitialized: Async<i32> = Async::default();
    assert!(!uninitialized.is_complete());
    assert!(uninitialized.should_load());
    assert!(uninitialized.is_incomplete());
    assert!(uninitialized.is_uninitialized());
    assert!(!uninitialized.is_loading());
    assert!(!uninitialized.is_success());
    assert!(!uninitialized.is_fail());
    assert!(!uninitialized.is_fail_with_timeout());
    assert!(!uninitialized.is_fail_with_none());
    assert!(!uninitialized.is_fail_with_error());
    assert!(!uninitialized.is_fail_with_canceled());
    assert!(uninitialized.value_ref().is_none());
    assert!(uninitialized.value().is_none());
}

#[test]
fn test_loading() {
    let loading = Async::loading(Some(7));
    assert!(!loading.is_complete());
    assert!(!loading.should_load());
    assert!(loading.is_incomplete());
    assert!(!loading.is_uninitialized());
    assert!(loading.is_loading());
    assert!(!loading.is_success());
    assert!(!loading.is_fail());
    assert!(!loading.is_fail_with_timeout());
    assert!(!loading.is_fail_with_none());
    assert!(!loading.is_fail_with_error());
    assert!(!loading.is_fail_with_canceled());
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
    assert!(success.is_complete());
    assert!(!success.should_load());
    assert!(!success.is_incomplete());
    assert!(!success.is_uninitialized());
    assert!(!success.is_loading());
    assert!(success.is_success());
    assert!(!success.is_fail());
    assert!(!success.is_fail_with_timeout());
    assert!(!success.is_fail_with_none());
    assert!(!success.is_fail_with_error());
    assert!(!success.is_fail_with_canceled());
    assert!(success.value_ref().is_some());
    assert_eq!(success.value_ref(), Some(&8));
    assert_eq!(success.value(), Some(8));
}

#[test]
fn test_fail() {
    let fail = Async::fail(AsyncError::Error("Connection failed".to_string()), Some(50));
    assert!(fail.is_complete());
    assert!(fail.should_load());
    assert!(!fail.is_incomplete());
    assert!(!fail.is_uninitialized());
    assert!(!fail.is_loading());
    assert!(!fail.is_success());
    assert!(fail.is_fail());
    assert!(fail.value_ref().is_some());
    assert_eq!(fail.value_ref(), Some(&50));
    assert_eq!(fail.value(), Some(50));
}

// Test factory methods
#[test]
fn test_fail_factory_methods() {
    // Test fail_with_error
    let fail = Async::fail_with_none(None::<i32>);
    assert_eq!(
        fail,
        Async::Fail {
            error: AsyncError::None,
            value: None
        }
    );
    // Test fail_with_cancelled
    let fail = Async::<i32>::fail_with_cancelled(Some(42));
    assert_eq!(
        fail,
        Async::Fail {
            error: AsyncError::Cancelled,
            value: Some(42)
        }
    );

    // Test fail_with_timeout
    let fail = Async::<i32>::fail_with_timeout(Some(42));
    assert_eq!(
        fail,
        Async::Fail {
            error: AsyncError::Timeout,
            value: Some(42)
        }
    );

    // Test fail_with_message
    let fail = Async::<i32>::fail_with_message("custom error", Some(42));
    assert_eq!(
        fail,
        Async::Fail {
            error: AsyncError::Error("custom error".to_string()),
            value: Some(42)
        }
    );
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

    // Test is_fail_with_timeout
    let fail = Async::fail(AsyncError::Timeout, None::<i32>);
    assert!(fail.is_fail_with_timeout());

    let fail = Async::fail(AsyncError::Error("error".to_string()), None::<i32>);
    assert!(!fail.is_fail_with_timeout());
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

// Test From trait implementation for &Async<T>
#[test]
fn test_retained_value_clone_from_async_ref() {
    // Test with Success
    let success = Async::success(42);
    let option: Option<i32> = (&success).value_ref_clone();
    assert_eq!(option, Some(42));

    // Test with Loading
    let loading = Async::loading(Some(42));
    let option: Option<i32> = (&loading).value_ref_clone();
    assert_eq!(option, Some(42));

    // Test with Loading (None)
    let loading = Async::loading(None::<i32>);
    let option: Option<i32> = (&loading).value_ref_clone();
    assert_eq!(option, None);

    // Test with Fail (with value)
    let fail = Async::fail(AsyncError::Error("error".to_string()), Some(42));
    let option: Option<i32> = (&fail).value_ref_clone();
    assert_eq!(option, Some(42));

    // Test with Fail (without value)
    let fail = Async::fail(AsyncError::Error("error".to_string()), None::<i32>);
    let option: Option<i32> = (&fail).value_ref_clone();
    assert_eq!(option, None);

    // Test with Uninitialized
    let uninitialized = Async::<i32>::Uninitialized;
    let option: Option<i32> = (&uninitialized).value_ref_clone();
    assert_eq!(option, None);
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
    assert!(state.is_complete());
    assert!(!state.should_load());
    assert!(!state.is_incomplete());
    assert_eq!(state.value_ref(), Some(&updated_user));

    // Transition to fail with retained value
    state = Async::fail(
        AsyncError::Error("Update failed".to_string()),
        Some(updated_user.clone()),
    );
    assert!(state.is_complete());
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
