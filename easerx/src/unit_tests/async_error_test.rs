use crate::AsyncError;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Test AsyncError methods
#[test]
fn test_async_error_methods() {
    let none_error = AsyncError::None;
    assert!(none_error.is_none());
    assert!(!none_error.is_error());
    assert!(!none_error.is_cancelled());
    assert!(!none_error.is_timeout());

    let error = AsyncError::error("message".to_string());
    assert!(!error.is_none());
    assert!(error.is_error());
    assert!(!error.is_cancelled());
    assert!(!error.is_timeout());

    let cancelled = AsyncError::Cancelled;
    assert!(!cancelled.is_none());
    assert!(!cancelled.is_error());
    assert!(cancelled.is_cancelled());
    assert!(!cancelled.is_timeout());

    let timeout = AsyncError::Timeout;
    assert!(!timeout.is_none());
    assert!(!timeout.is_error());
    assert!(!timeout.is_cancelled());
    assert!(timeout.is_timeout());
}

#[cfg(feature = "serde")]
#[test]
fn test_async_error_serde() {
    use serde_json;

    let error = AsyncError::error("message");
    let serialized = serde_json::to_string(&error).unwrap();
    assert_eq!(serialized, r#"{"error":"message"}"#);

    let deserialized: AsyncError = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, error);

    let none_error = AsyncError::None;
    let serialized_none = serde_json::to_string(&none_error).unwrap();
    assert_eq!(serialized_none, r#""none""#);

    let deserialized_none: AsyncError = serde_json::from_str(&serialized_none).unwrap();
    assert_eq!(deserialized_none, none_error);

    let cancelled_error = AsyncError::Cancelled;
    let serialized_cancelled = serde_json::to_string(&cancelled_error).unwrap();
    assert_eq!(serialized_cancelled, r#""cancelled""#);

    let deserialized_none: AsyncError = serde_json::from_str(&serialized_cancelled).unwrap();
    assert_eq!(deserialized_none, cancelled_error);

    let cancelled_timeout = AsyncError::Timeout;
    let serialized_timeout = serde_json::to_string(&cancelled_timeout).unwrap();
    assert_eq!(serialized_timeout, r#""timeout""#);

    let deserialized_none: AsyncError = serde_json::from_str(&serialized_timeout).unwrap();
    assert_eq!(deserialized_none, cancelled_timeout);
}
#[test]
fn test_async_error_marco_debug(){
    let error = AsyncError::error("message");
    let debug_str = format!("{:?}", error);
    assert_eq!(debug_str, r#"Error("message")"#);
    
    let none_error = AsyncError::None;
    let debug_none_str = format!("{:?}", none_error);
    assert_eq!(debug_none_str, r#"None"#);
    
    let cancelled_error = AsyncError::Cancelled;
    let debug_cancelled_str = format!("{:?}", cancelled_error);
    assert_eq!(debug_cancelled_str, r#"Cancelled"#);
    
    let timeout_error = AsyncError::Timeout;
    let debug_timeout_str = format!("{:?}", timeout_error);
    assert_eq!(debug_timeout_str, r#"Timeout"#);
}
#[test]
fn test_async_error_hash() {
    let err1 = AsyncError::error("message".to_string());
    let err2 = AsyncError::error("message".to_string());
    let err1_hash = {
        let mut hasher = DefaultHasher::new();
        err1.hash(&mut hasher);
        hasher.finish()
    };
    let err2_hash = {
        let mut hasher = DefaultHasher::new();
        err2.hash(&mut hasher);
        hasher.finish()
    };

    assert_eq!(err1_hash, err2_hash);

    let none1 = AsyncError::None;
    let none2 = AsyncError::None;
    let non1_hash = {
        let mut hasher = DefaultHasher::new();
        none1.hash(&mut hasher);
        hasher.finish()
    };
    let non2_hash = {
        let mut hasher = DefaultHasher::new();
        none2.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(non1_hash, non2_hash);

    let cancelled1 = AsyncError::Cancelled;
    let cancelled2 = AsyncError::Cancelled;
    let cancelled1_hash = {
        let mut hasher = DefaultHasher::new();
        cancelled1.hash(&mut hasher);
        hasher.finish()
    };
    let cancelled2_hash = {
        let mut hasher = DefaultHasher::new();
        cancelled2.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(cancelled1_hash, cancelled2_hash);

    let timeout1 = AsyncError::Timeout;
    let timeout2 = AsyncError::Timeout;
    let timeout1_hash = {
        let mut hasher = DefaultHasher::new();
        timeout1.hash(&mut hasher);
        hasher.finish()
    };
    let timeout2_hash = {
        let mut hasher = DefaultHasher::new();
        timeout2.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(timeout1_hash, timeout2_hash);

    assert_ne!(err1_hash, non1_hash);
    assert_ne!(err1_hash, cancelled1_hash);
    assert_ne!(err1_hash, timeout1_hash);
}
