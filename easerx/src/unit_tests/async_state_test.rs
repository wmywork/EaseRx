use std::hash::{Hash, Hasher};
use crate::async_error::AsyncError;
use crate::Async;

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
    let fail = Async::fail(AsyncError::error("Connection failed"), Some(50));
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

#[cfg(feature = "serde")]
#[test]
fn test_async_state_serde() {
    use serde_json;

    let uninitialized = Async::<i32>::Uninitialized;
    let serialized_uninitialized = serde_json::to_string(&uninitialized).unwrap();
    assert_eq!(serialized_uninitialized, r#""uninitialized""#);

    let success = Async::success(42);
    let serialized = serde_json::to_string(&success).unwrap();
    assert_eq!(serialized, r#"{"success":{"value":42}}"#);

    let deserialized: Async<i32> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, success);

    let loading = Async::loading(Some(42));
    let serialized_loading = serde_json::to_string(&loading).unwrap();
    assert_eq!(serialized_loading, r#"{"loading":{"value":42}}"#);

    let deserialized_loading: Async<i32> = serde_json::from_str(&serialized_loading).unwrap();
    assert_eq!(deserialized_loading, loading);

    let fail_err = Async::fail(AsyncError::error("test"), Some(42));
    let serialized_fail_err = serde_json::to_string(&fail_err).unwrap();
    assert_eq!(
        serialized_fail_err,
        r#"{"fail":{"error":{"error":"test"},"value":42}}"#
    );

    let deserialized_fail: Async<i32> = serde_json::from_str(&serialized_fail_err).unwrap();
    assert_eq!(deserialized_fail, fail_err);

    let fail_none = Async::fail(AsyncError::None, Some(42));
    let serialized_fail_none = serde_json::to_string(&fail_none).unwrap();
    assert_eq!(
        serialized_fail_none,
        r#"{"fail":{"error":"none","value":42}}"#
    );

    let deserialized_fail: Async<i32> = serde_json::from_str(&serialized_fail_none).unwrap();
    assert_eq!(deserialized_fail, fail_none);

    let fail_cancelled = Async::fail(AsyncError::Cancelled, Some(42));
    let serialized_fail_cancelled = serde_json::to_string(&fail_cancelled).unwrap();
    assert_eq!(
        serialized_fail_cancelled,
        r#"{"fail":{"error":"cancelled","value":42}}"#
    );

    let deserialized_fail: Async<i32> = serde_json::from_str(&serialized_fail_cancelled).unwrap();
    assert_eq!(deserialized_fail, fail_cancelled);

    let fail_timeout = Async::fail(AsyncError::Timeout, Some(42));
    let serialized_fail_timeout = serde_json::to_string(&fail_timeout).unwrap();
    assert_eq!(
        serialized_fail_timeout,
        r#"{"fail":{"error":"timeout","value":42}}"#
    );

    let deserialized_fail: Async<i32> = serde_json::from_str(&serialized_fail_timeout).unwrap();
    assert_eq!(deserialized_fail, fail_timeout);
}

#[test]
fn test_async_state_marco_debug() {
    let uninitialized: Async<i32> = Async::default();
    let debug_str = format!("{:?}", uninitialized);
    assert_eq!(debug_str, "Uninitialized");
    
    let loading = Async::loading(Some(42));
    let debug_str = format!("{:?}", loading);
    assert_eq!(debug_str, "Loading { value: Some(42) }");
    
    let success = Async::success(42);
    let debug_str = format!("{:?}", success);
    assert_eq!(debug_str, "Success { value: 42 }");
    
    let fail = Async::fail(AsyncError::error("test"), Some(42));
    let debug_str = format!("{:?}", fail);
    assert_eq!(debug_str, "Fail { error: Error(\"test\"), value: Some(42) }");
}
#[test]
fn test_async_state_hash(){

    let uninitialized1: Async<i32> = Async::default();
    let uninitialized2: Async<i32> = Async::default();
    let unintialized1_hash={
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        uninitialized1.hash(&mut hasher);
        hasher.finish()
    };
    let unintialized2_hash={
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        uninitialized2.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(unintialized1_hash, unintialized2_hash);

    let loading1 = Async::loading(Some(42));
    let loading2 = Async::loading(Some(42));
    let loading1_hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        loading1.hash(&mut hasher);
        hasher.finish()
    };
    let loading2_hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        loading2.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(loading1_hash, loading2_hash);

    let success1 = Async::success(42);
    let success2 = Async::success(42);
    let success1_hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        success1.hash(&mut hasher);
        hasher.finish()
    };
    let success2_hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        success2.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(success1_hash, success2_hash);

    let fail1 = Async::fail(AsyncError::error("test"), Some(42));
    let fail2 = Async::fail(AsyncError::error("test"), Some(42));
    let fail1_hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        fail1.hash(&mut hasher);
        hasher.finish()
    };
    let fail2_hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        fail2.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(fail1_hash, fail2_hash);

    assert_ne!(unintialized1_hash,loading1_hash);
    assert_ne!(unintialized1_hash, success1_hash);
    assert_ne!(unintialized1_hash, fail1_hash);
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
            error: AsyncError::error("custom error"),
            value: Some(42)
        }
    );
}

// Test error state helpers
#[test]
fn test_error_state_helpers() {
    // Test is_fail_with_error
    let fail = Async::fail(AsyncError::error("error"), None::<i32>);
    assert!(fail.is_fail_with_error());

    let fail = Async::fail(AsyncError::None, None::<i32>);
    assert!(!fail.is_fail_with_error());

    // Test is_fail_with_none
    let fail = Async::fail(AsyncError::None, None::<i32>);
    assert!(fail.is_fail_with_none());

    let fail = Async::fail(AsyncError::error("error"), None::<i32>);
    assert!(!fail.is_fail_with_none());

    // Test is_fail_with_canceled
    let fail = Async::fail(AsyncError::Cancelled, None::<i32>);
    assert!(fail.is_fail_with_canceled());

    let fail = Async::fail(AsyncError::error("error"), None::<i32>);
    assert!(!fail.is_fail_with_canceled());

    // Test is_fail_with_timeout
    let fail = Async::fail(AsyncError::Timeout, None::<i32>);
    assert!(fail.is_fail_with_timeout());

    let fail = Async::fail(AsyncError::error("error"), None::<i32>);
    assert!(!fail.is_fail_with_timeout());
}

#[test]
fn test_value_ref_clone_from_async_ref() {
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
    let fail = Async::fail(AsyncError::error("error"), Some(42));
    let option: Option<i32> = (&fail).value_ref_clone();
    assert_eq!(option, Some(42));

    // Test with Fail (without value)
    let fail = Async::fail(AsyncError::error("error"), None::<i32>);
    let option: Option<i32> = (&fail).value_ref_clone();
    assert_eq!(option, None);

    // Test with Uninitialized
    let uninitialized = Async::<i32>::Uninitialized;
    let option: Option<i32> = (&uninitialized).value_ref_clone();
    assert_eq!(option, None);
}

#[test]
fn test_set_retain_value() {
    // Test with Loading
    let mut loading = Async::loading(Some(42));
    loading = loading.set_retain_value(Some(100));
    assert!(loading.is_loading());
    assert_eq!(loading.value_ref(), Some(&100));

    // Test with Fail
    let mut fail = Async::fail(AsyncError::error("error"), Some(42));
    fail = fail.set_retain_value(Some(100));
    assert!(fail.is_fail());
    assert_eq!(fail.value_ref(), Some(&100));

    // Test with Success
    let mut success = Async::success(42);
    success = success.set_retain_value(Some(100));
    assert!(success.is_success());
    assert_eq!(success.value_ref(), Some(&42));

    // Test with Uninitialized
    let mut uninitialized: Async<i32> = Async::default();
    uninitialized = uninitialized.set_retain_value(Some(100));
    assert!(uninitialized.is_uninitialized());
    assert_eq!(uninitialized.value_ref(), None);
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
        AsyncError::error("Update failed"),
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
