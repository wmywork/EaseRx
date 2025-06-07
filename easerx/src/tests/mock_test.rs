use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tokio::time::sleep;
use crate::{Async, AsyncError, State};
use crate::mock::{assert, MockStateStore};

#[derive(Debug, Clone, PartialEq)]
struct TestState {
    counter: i32,
    data: Option<String>,
}

impl State for TestState {}

#[tokio::test]
async fn test_mock_state_store() {
    let initial_state = TestState {
        counter: 0,
        data: None,
    };

    let mock_store = MockStateStore::new(initial_state);

    // 测试设置状态
    mock_store.set_state(|state| TestState {
        counter: state.counter + 1,
        ..state
    });

    let state = mock_store.get_state();
    assert_eq!(state.counter, 1);

    // 测试执行操作
    mock_store.mock_result(Async::Success {
        value: "test data".to_string(),
    });

    mock_store
        .execute(|state, result| {
            let data = result.value().unwrap_or_default();
            TestState {
                data: Some(data),
                ..state
            }
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, Some("test data".to_string()));

    // 测试操作历史
    let operations = mock_store.get_operations::<String>();
    assert_eq!(operations.len(), 1);
}

#[tokio::test]
async fn test_mock_with_cancellation() {
    let initial_state = TestState {
        counter: 0,
        data: None,
    };

    let mut mock_store = MockStateStore::new(initial_state);
    mock_store.set_delay(Duration::from_millis(100));

    let token = CancellationToken::new();
    let token_clone = token.clone();

    // 在另一个任务中取消操作
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        token_clone.cancel();
    });

    mock_store
        .execute_cancellable::<bool, _>(token, |state, result| {
            assert!(result.is_fail_with_canceled());
            state
        })
        .await;
}

#[tokio::test]
async fn test_mock_with_timeout() {
    let initial_state = TestState {
        counter: 0,
        data: None,
    };

    let mut mock_store = MockStateStore::new(initial_state);
    mock_store.set_delay(Duration::from_millis(200));

    mock_store
        .execute_with_timeout::<bool, _>(Duration::from_millis(100), |state, result| {
            assert!(matches!(
                result,
                Async::Fail {
                    error: AsyncError::Timeout,
                    ..
                }
            ));
            state
        })
        .await;
}

#[tokio::test]
async fn test_assert_helpers() {
    let initial_state = TestState {
        counter: 0,
        data: None,
    };
    let mock_store = MockStateStore::new(initial_state.clone());

    // 更新状态
    mock_store.set_state(|_| TestState {
        counter: 1,
        data: Some("测试数据".to_string()),
    });

    // 测试状态断言
    assert::assert_state(&mock_store, TestState {
        counter: 1,
        data: Some("测试数据".to_string()),
    });

    // 测试操作数量断言
    assert::assert_operation_count::<_, ()>(&mock_store, 1);

    // 清除操作历史
    mock_store.clear_operations();

    // 模拟执行
    mock_store.mock_result(Async::Success { value: "执行结果".to_string() });
    mock_store.execute::<String, _>(|state, result| {
        assert_eq!(result, Async::Success { value: "执行结果".to_string() });
        TestState {
            data: Some(result.value().unwrap_or_default()),
            ..state
        }
    }).await;

    // 测试执行结果断言
    assert::assert_execution_result(&mock_store, Async::Success { value: "执行结果".to_string() });
}

#[tokio::test]
async fn test_mock_http_client() {
    use crate::mock::network::*;

    let mut client = MockHttpClient::new();

    // 预设响应
    client.mock_response(
        "https://example.com",
        MockHttpResponse::new(200, "Hello, World!")
            .with_header("Content-Type", "text/plain"),
    );

    // 执行请求
    let response = client.get("https://example.com").await.unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"Hello, World!");
    assert_eq!(
        response.headers.get("Content-Type").unwrap(),
        "text/plain"
    );

    // 测试未预设的URL
    let error = client.get("https://unknown.com").await.unwrap_err();
    assert!(error.contains("没有为URL"));
}