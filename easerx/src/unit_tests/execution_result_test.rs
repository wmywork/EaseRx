use crate::{Async, ExecutionResult};

#[test]
fn test_value_to_async() {
    let value = 42;
    let async_value = value.into_async();

    assert_eq!(async_value, Async::Success { value: 42 });
}

#[test]
fn test_result_ok_to_async() {
    let result: Result<i32, &str> = Ok(42);
    let async_result = result.into_async();

    assert_eq!(async_result, Async::Success { value: 42 });
}

#[test]
fn test_result_err_to_async() {
    let result: Result<i32, &str> = Err("error message");
    let async_result: Async<i32> = result.into_async();

    assert_eq!(
        async_result,
        Async::fail_with_message("error message".to_string(), None)
    );
}

#[test]
fn test_option_some_to_async() {
    let option = Some(42);
    let async_option = option.into_async();

    assert_eq!(async_option, Async::Success { value: 42 });
}

#[test]
fn test_option_none_to_async() {
    let option: Option<i32> = None;
    let async_option: Async<i32> = option.into_async();

    assert_eq!(async_option, Async::fail_with_none(None));
}

#[test]
fn test_complex_type_conversion() {
    #[derive(Clone, Debug, PartialEq)]
    struct User {
        id: i32,
        name: String,
    }

    let user = User {
        id: 1,
        name: "John".to_string(),
    };
    let async_user = user.clone().into_async();

    assert_eq!(async_user, Async::success(user));
}

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

    assert_eq!(
        async_result,
        Async::fail_with_message("custom error".to_string(), None)
    );
}
