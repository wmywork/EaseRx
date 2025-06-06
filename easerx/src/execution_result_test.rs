use crate::{Async, AsyncError, ExecutionResult};

#[cfg(test)]
mod tests {
    use super::*;

    // Test direct value conversion to Async::Success
    #[test]
    fn test_value_to_async() {
        let value = 42;
        let async_value = value.into_async();

        assert!(matches!(async_value, Async::Success { value: 42 }));
    }

    // Test Result::Ok conversion to Async::Success
    #[test]
    fn test_result_ok_to_async() {
        let result: Result<i32, &str> = Ok(42);
        let async_result = result.into_async();

        assert!(matches!(async_result, Async::Success { value: 42 }));
    }

    // Test Result::Err conversion to Async::Fail
    #[test]
    fn test_result_err_to_async() {
        let result: Result<i32, &str> = Err("error message");
        let async_result = result.into_async();

        match async_result {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Error(msg) if msg == "error message"));
                assert!(value.is_none());
            }
            _ => panic!("Expected Async::Fail variant"),
        }
    }

    // Test Option::Some conversion to Async::Success
    #[test]
    fn test_option_some_to_async() {
        let option = Some(42);
        let async_option = option.into_async();

        assert!(matches!(async_option, Async::Success { value: 42 }));
    }

    // Test Option::None conversion to Async::Fail with None error
    #[test]
    fn test_option_none_to_async() {
        let option: Option<i32> = None;
        let async_option = option.into_async();

        match async_option {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::None));
                assert!(value.is_none());
            }
            _ => panic!("Expected Async::Fail variant"),
        }
    }

    // Test complex types conversion
    #[test]
    fn test_complex_type_conversion() {
        #[derive(Clone)]
        struct User {
            id: i32,
            name: String,
        }

        let user = User {
            id: 1,
            name: "John".to_string(),
        };
        let async_user = user.into_async();

        match async_user {
            Async::Success { value } => {
                assert_eq!(value.id, 1);
                assert_eq!(value.name, "John");
            }
            _ => panic!("Expected Async::Success variant"),
        }
    }

    // Test Result with custom error type
    #[test]
    fn test_result_with_custom_error() {
        #[derive(Debug)]
        struct CustomError(String);

        impl ToString for CustomError {
            fn to_string(&self) -> String {
                self.0.clone()
            }
        }

        let result: Result<i32, CustomError> = Err(CustomError("custom error".to_string()));
        let async_result = result.into_async();

        match async_result {
            Async::Fail { error, value } => {
                assert!(matches!(error, AsyncError::Error(msg) if msg == "custom error"));
                assert!(value.is_none());
            }
            _ => panic!("Expected Async::Fail variant"),
        }
    }
}
