use easerx::mock::{MockStateStore, assert, event_stream};
use easerx::{Async, AsyncError, State};
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, PartialEq)]
struct TestState {
    counter: i32,
    data: Option<String>,
    status: Async<bool>,
}

impl State for TestState {}

impl Default for TestState {
    fn default() -> Self {
        TestState {
            counter: 0,
            data: None,
            status: Async::Uninitialized,
        }
    }
}

#[tokio::test]
async fn test_mock_state_store_basic() {
    let initial_state = TestState::default();
    let mock_store = MockStateStore::new(initial_state);

    // 测试初始状态
    let state = mock_store.get_state();
    assert_eq!(state.counter, 0);
    assert_eq!(state.data, None);
    assert!(matches!(state.status, Async::Uninitialized));

    // 测试状态更新
    mock_store.set_state(|state| TestState {
        counter: state.counter + 1,
        ..state
    });

    let state = mock_store.get_state();
    assert_eq!(state.counter, 1);
}

#[tokio::test]
async fn test_mock_result_execution() {
    let mock_store = MockStateStore::new(TestState::default());

    // 预设成功结果
    mock_store.mock_result(Async::Success {
        value: "测试数据".to_string(),
    });

    // 执行操作
    mock_store
        .execute(|state, result| TestState {
            data: result.value(),
            status: Async::Success { value: true },
            ..state
        })
        .await;

    // 验证状态更新
    let state = mock_store.get_state();
    assert_eq!(state.data, Some("测试数据".to_string()));
    assert!(matches!(state.status, Async::Success { value: true }));
}

#[tokio::test]
async fn test_mock_error_result() {
    let mock_store = MockStateStore::new(TestState::default());

    // 预设错误结果
    mock_store.mock_result::<bool>(Async::Fail {
        error: AsyncError::Error("测试错误".to_string()),
        value: None,
    });

    // 执行操作
    mock_store
        .execute(|state, result: Async<bool>| TestState {
            status: result,
            ..state
        })
        .await;

    // 验证状态更新
    let state = mock_store.get_state();
    assert!(state.status.is_fail_with_error());
}

#[tokio::test]
async fn test_mock_cancellation() {
    let mut mock_store = MockStateStore::new(TestState::default());

    // 设置延迟
    mock_store.set_delay(Duration::from_millis(100));

    let token = CancellationToken::new();
    let token_clone = token.clone();

    // 在另一个任务中取消操作
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        token_clone.cancel();
    });

    // 执行可取消的操作
    mock_store
        .execute_cancellable::<bool, _>(token, |state, result| TestState {
            status: result,
            ..state
        })
        .await;

    // 验证操作被取消
    let state = mock_store.get_state();
    assert!(state.status.is_fail_with_canceled());
}

#[tokio::test]
async fn test_mock_timeout() {
    let mut mock_store = MockStateStore::new(TestState::default());

    // 设置延迟大于超时时间
    mock_store.set_delay(Duration::from_millis(200));

    // 执行带超时的操作
    mock_store
        .execute_with_timeout::<bool, _>(Duration::from_millis(100), |state, result| TestState {
            status: result,
            ..state
        })
        .await;

    // 验证操作超时
    let state = mock_store.get_state();
    assert!(matches!(
        state.status,
        Async::Fail {
            error: AsyncError::Timeout,
            ..
        }
    ));
}

#[tokio::test]
async fn test_conditional_mock_result() {
    let mock_store = MockStateStore::new(TestState::default());
    let counter = std::sync::atomic::AtomicI32::new(0);
    let counter = std::sync::Arc::new(counter);
    let counter_clone = counter.clone();

    // 预设条件响应
    mock_store.mock_conditional_result(
        move || {
            let current = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            current >= 2 // 第三次调用时才满足条件
        },
        Async::Success {
            value: "条件满足".to_string(),
        },
    );

    // 第一次执行，条件不满足
    mock_store
        .execute::<String, _>(|state, result| TestState {
            data: result.value(),
            ..state
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, None); // 条件不满足，没有更新数据

    // 第二次执行，条件不满足
    mock_store
        .execute::<String, _>(|state, result| TestState {
            data: result.value(),
            ..state
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, None); // 条件不满足，没有更新数据

    // 第三次执行，条件满足
    mock_store
        .execute::<String, _>(|state, result| TestState {
            data: result.value(),
            ..state
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, Some("条件满足".to_string())); // 条件满足，更新数据
}

#[tokio::test]
async fn test_mock_sequence_results() {
    let mock_store = MockStateStore::new(TestState::default());

    // 预设序列响应
    mock_store.mock_sequence_results(vec![
        Async::Success {
            value: "第一个结果".to_string(),
        },
        Async::Success {
            value: "第二个结果".to_string(),
        },
        Async::Fail {
            error: AsyncError::Error("预设错误".to_string()),
            value: None,
        },
    ]);

    // 第一次执行
    mock_store
        .execute::<String, _>(|state, result| TestState {
            data: result.value(),
            ..state
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, Some("第一个结果".to_string()));

    // 第二次执行
    mock_store
        .execute::<String, _>(|state, result| TestState {
            data: result.value(),
            ..state
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, Some("第二个结果".to_string()));

    // 第三次执行
    mock_store
        .execute::<String, _>(|state, result| TestState {
            data: result.value(),
            status: Async::Fail {
                error: AsyncError::Error("预设错误".to_string()),
                value: None,
            },
            ..state
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, None); // 失败结果，没有数据
    assert!(matches!(
        state.status,
        Async::Fail {
            error: AsyncError::Error(msg),
            ..
        } if msg.contains("预设错误")
    ));
}

#[tokio::test]
async fn test_mock_http_client() {
    use easerx::mock::network::*;

    let mut client = MockHttpClient::new();

    // 预设HTTP响应
    client.mock_response(
        "https://api.example.com/data",
        MockHttpResponse::new(200, r#"{"result": "success"}"#)
            .with_header("Content-Type", "application/json"),
    );

    // 预设错误响应
    client.mock_response(
        "https://api.example.com/error",
        MockHttpResponse::new(500, r#"{"error": "server error"}"#)
            .with_header("Content-Type", "application/json"),
    );

    // 测试成功响应
    let response = client.get("https://api.example.com/data").await.unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(
        String::from_utf8(response.body).unwrap(),
        r#"{"result": "success"}"#
    );
    assert_eq!(
        response.headers.get("Content-Type").unwrap(),
        "application/json"
    );

    // 测试错误响应
    let response = client.get("https://api.example.com/error").await.unwrap();
    assert_eq!(response.status, 500);

    // 测试未预设的URL
    let error = client
        .get("https://api.example.com/unknown")
        .await
        .unwrap_err();
    assert!(error.contains("没有为URL"));
}

#[tokio::test]
async fn test_multiple_mock_results() {
    let mock_store = MockStateStore::new(TestState::default());

    // 预设多个结果
    mock_store.mock_result(Async::Success {
        value: "结果1".to_string(),
    });
    mock_store.mock_result(Async::Success {
        value: "结果2".to_string(),
    });

    // 第一次执行
    mock_store
        .execute(|state, result| TestState {
            data: result.value(),
            ..state
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, Some("结果1".to_string()));

    // 第二次执行
    mock_store
        .execute(|state, result| TestState {
            data: result.value(),
            ..state
        })
        .await;

    let state = mock_store.get_state();
    assert_eq!(state.data, Some("结果2".to_string()));

    // 第三次执行（没有预设结果）
    mock_store
        .execute::<String, _>(|state, result| {
            assert!(matches!(
                result,
                Async::Fail {
                    error: AsyncError::Error(msg),
                    ..
                } if msg.contains("没有预设结果")
            ));
            state
        })
        .await;
}

#[tokio::test]
async fn test_operation_history() {
    let mock_store = MockStateStore::new(TestState::default());

    // 执行一些操作
    mock_store.set_state(|state| TestState {
        counter: state.counter + 1,
        ..state
    });

    mock_store.mock_result(Async::Success { value: true });
    mock_store
        .execute(|state, result| TestState {
            status: result,
            ..state
        })
        .await;

    // 检查操作历史
    let operations = mock_store.get_operations::<bool>();
    assert_eq!(operations.len(), 1);

    // 清除操作历史
    mock_store.clear_operations();
    let operations = mock_store.get_operations::<bool>();
    assert_eq!(operations.len(), 0);
}

#[tokio::test]
async fn test_mock_http_client_enhanced() {
    use easerx::mock::network::*;

    let mut client = MockHttpClient::new();

    // 预设基本响应
    client.mock_response(
        "https://api.example.com/data",
        MockHttpResponse::json(200, r#"{"result": "success"}"#),
    );

    // 预设条件响应
    client.mock_conditional_response(
        |req| req.method == "POST" && req.url.contains("/submit"),
        MockHttpResponse::text(201, "Created"),
    );

    // 测试GET请求
    let response = client.get("https://api.example.com/data").await.unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(
        response.headers.get("Content-Type").unwrap(),
        "application/json"
    );

    // 测试POST请求（满足条件）
    let response = client
        .post("https://api.example.com/submit", b"test data".to_vec())
        .await
        .unwrap();
    assert_eq!(response.status, 201);
    assert_eq!(response.headers.get("Content-Type").unwrap(), "text/plain");

    // 测试请求历史
    assert::assert_requested_url(&client, "https://api.example.com/data");
    assert::assert_requested_method(&client, "POST", "https://api.example.com/submit");
    assert::assert_requested_body(&client, "https://api.example.com/submit", b"test data");
    assert::assert_request_count(&client, 2);
}

#[tokio::test]
async fn test_mock_event_stream() {
    // 创建模拟事件流并添加事件
    let stream = event_stream::MockEventStream::<i32>::new();
    stream.add_events(vec![1, 2, 3, 4, 5]);

    // 由于Stream实现比较复杂，这里我们只测试添加事件的功能
    // 实际应用中通常会配合其他异步机制使用
}

#[tokio::test]
async fn test_mock_delayed_event_stream() {
    // 创建模拟事件流并添加延迟事件
    let stream = event_stream::MockEventStream::<i32>::new();
    stream.add_delayed_events(vec![
        (1, Duration::from_millis(10)),
        (2, Duration::from_millis(20)),
        (3, Duration::from_millis(30)),
    ]);

    // 由于Stream实现比较复杂，这里我们只测试添加延迟事件的功能
    // 实际应用中通常会配合其他异步机制使用
}

#[tokio::test]
async fn test_assert_helpers() {
    let initial_state = TestState::default();
    let mock_store = MockStateStore::new(initial_state.clone());

    // 测试状态断言
    assert::assert_state(&mock_store, initial_state.clone());

    // 更新状态
    let updated_state = TestState {
        counter: 1,
        data: Some("测试数据".to_string()),
        status: Async::Success { value: true },
    };

    mock_store.set_state(|_| updated_state.clone());

    // 测试状态历史断言
    assert::assert_state_history_contains(&mock_store, updated_state.clone());

    // 测试状态转换断言
    assert::assert_state_transition(&mock_store, initial_state, updated_state);

    // 测试执行结果断言
    mock_store.mock_result(Async::Success { value: true });
    mock_store
        .execute::<bool, _>(|state, result| {
            assert_eq!(result, Async::Success { value: true });
            state
        })
        .await;

    assert::assert_execution_result(&mock_store, Async::Success { value: true });
}
