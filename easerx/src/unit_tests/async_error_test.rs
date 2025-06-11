use crate::AsyncError;

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
    assert_eq!(serialized, r#"{"any":{"msg":"message"}}"#);

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
