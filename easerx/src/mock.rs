use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use crate::{Async, AsyncError, State};

/// 记录对状态存储的操作历史
#[derive(Debug, Clone, PartialEq)]
pub enum StateOperation<S: State + PartialEq, T: Clone + PartialEq + Send + 'static> {
    /// 状态更新操作
    StateUpdate {
        /// 操作前的状态
        old_state: S,
        /// 操作后的状态
        new_state: S,
    },
    /// 执行操作
    Execute {
        /// 操作结果
        result: Async<T>,
    },
}

/// 预设结果类型
enum MockedResult<T: Clone + PartialEq + Send + 'static> {
    /// 普通结果
    Normal(Async<T>),
    /// 条件结果
    Conditional(Box<dyn Fn() -> bool + Send>, Async<T>),
}

/// Mock状态存储，用于测试
pub struct MockStateStore<S: State> {
    /// 内部状态
    state: Arc<Mutex<S>>,
    /// 操作历史记录
    operations: Arc<Mutex<Vec<Box<dyn std::any::Any + Send + 'static>>>>,
    /// 预设的执行结果队列
    mocked_results: Arc<Mutex<VecDeque<Box<dyn std::any::Any + Send + 'static>>>>,
    /// 预设的延迟时间
    delay: Option<Duration>,
}

impl<S: State> MockStateStore<S> {
    /// 创建新的Mock状态存储
    pub fn new(initial_state: S) -> Self {
        MockStateStore {
            state: Arc::new(Mutex::new(initial_state)),
            operations: Arc::new(Mutex::new(Vec::new())),
            mocked_results: Arc::new(Mutex::new(VecDeque::new())),
            delay: None,
        }
    }

    /// 获取当前状态
    pub fn get_state(&self) -> S {
        self.state.lock().unwrap().clone()
    }

    /// 设置状态
    pub fn set_state<F>(&self, reducer: F)
    where
        F: FnOnce(S) -> S,
        S: PartialEq,
    {
        let mut state = self.state.lock().unwrap();
        let old_state = state.clone();
        let new_state = reducer(state.clone());
        *state = new_state.clone();

        let operation: StateOperation<S, ()> = StateOperation::StateUpdate {
            old_state,
            new_state,
        };
        self.record_operation(operation);
    }

    /// 记录操作历史
    fn record_operation<T: Clone + PartialEq + Send + 'static>(&self, operation: StateOperation<S, T>) 
    where S: PartialEq
    {
        let mut operations = self.operations.lock().unwrap();
        operations.push(Box::new(operation));
    }

    /// 预设执行结果
    pub fn mock_result<T: Clone + PartialEq + Send + 'static>(&self, result: Async<T>) {
        let mut results = self.mocked_results.lock().unwrap();
        results.push_back(Box::new(MockedResult::Normal(result)));
    }

    /// 预设条件响应结果
    pub fn mock_conditional_result<T, F>(&self, condition: F, result: Async<T>)
    where
        T: Clone + PartialEq + Send + 'static,
        F: Fn() -> bool + Send + 'static + 'static,
    {
        let mut results = self.mocked_results.lock().unwrap();
        results.push_back(Box::new(MockedResult::Conditional(Box::new(condition), result)));
    }

    /// 预设序列响应结果
    pub fn mock_sequence_results<T>(&self, results: Vec<Async<T>>)
    where
        T: Clone + PartialEq + Send + 'static,
    {
        let mut mocked_results = self.mocked_results.lock().unwrap();
        for result in results {
            mocked_results.push_back(Box::new(MockedResult::Normal(result)));
        }
    }

    /// 获取下一个预设结果
    fn next_result<T: Clone + PartialEq + Send + 'static>(&self) -> Async<T> {
        let mut results = self.mocked_results.lock().unwrap();
        if let Some(result) = results.pop_front() {
            // 尝试转换为MockedResult类型
            if let Ok(mocked_result) = result.downcast::<MockedResult<T>>() {
                match *mocked_result {
                    MockedResult::Normal(result) => result,
                    MockedResult::Conditional(condition, result) => {
                        if condition() {
                            result
                        } else {
                            // 条件不满足，将结果放回队列前端
                            results.push_front(Box::new(MockedResult::Conditional(condition, result)));
                            Async::Fail {
                                error: AsyncError::Error("条件不满足".to_string()),
                                value: None,
                            }
                        }
                    }
                }
            } else {
                Async::Fail {
                    error: AsyncError::Error("类型不匹配".to_string()),
                    value: None,
                }
            }
        } else {
            Async::Fail {
                error: AsyncError::Error("没有预设结果".to_string()),
                value: None,
            }
        }
    }

    /// 设置模拟延迟
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = Some(delay);
    }

    /// 获取操作历史
    pub fn get_operations<T: Clone + PartialEq + Send + 'static>(&self) -> Vec<StateOperation<S, T>> 
    where S: PartialEq
    {
        let operations = self.operations.lock().unwrap();
        operations
            .iter()
            .filter_map(|op| {
                op.downcast_ref::<StateOperation<S, T>>().cloned()
            })
            .collect()
    }

    /// 清除操作历史
    pub fn clear_operations(&self) {
        let mut operations = self.operations.lock().unwrap();
        operations.clear();
    }

    /// 执行模拟操作
    pub async fn execute<T, U>(&self, state_updater: U)
    where
        T: Clone + PartialEq + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        S: PartialEq,
    {
        // 应用预设的延迟
        if let Some(delay) = self.delay {
            sleep(delay).await;
        }

        // 获取预设的结果
        let result = self.next_result::<T>();

        // 记录执行操作
        let operation = StateOperation::Execute {
            result: result.clone(),
        };
        self.record_operation(operation);

        // 更新状态
        self.set_state(|old_state| state_updater(old_state, result));
    }

    /// 执行可取消的模拟操作
    pub async fn execute_cancellable<T, U>(
        &self,
        cancellation_token: CancellationToken,
        state_updater: U,
    ) where
        T: Clone + PartialEq + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        S: PartialEq,
    {
        // 应用预设的延迟
        if let Some(delay) = self.delay {
            tokio::select! {
                _ = sleep(delay) => {},
                _ = cancellation_token.cancelled() => {
                    let result = Async::fail_with_cancelled(None);
                    
                    // 记录执行操作
                    let operation = StateOperation::Execute {
                        result: result.clone(),
                    };
                    self.record_operation(operation);
                    
                    // 更新状态
                    self.set_state(|old_state| state_updater(old_state, result));
                    return;
                }
            }
        }

        // 获取预设的结果
        let result = self.next_result::<T>();

        // 记录执行操作
        let operation = StateOperation::Execute {
            result: result.clone(),
        };
        self.record_operation(operation);

        // 更新状态
        self.set_state(|old_state| state_updater(old_state, result));
    }

    /// 执行带超时的模拟操作
    pub async fn execute_with_timeout<T, U>(
        &self,
        timeout: Duration,
        state_updater: U,
    ) where
        T: Clone + PartialEq + Send + 'static,
        U: FnOnce(S, Async<T>) -> S + Clone + Send + 'static,
        S: PartialEq,
    {
        // 应用预设的延迟
        if let Some(delay) = self.delay {
            if delay > timeout {
                let result = Async::fail_with_timeout(None);
                
                // 记录执行操作
                let operation = StateOperation::Execute {
                    result: result.clone(),
                };
                self.record_operation(operation);
                
                // 更新状态
                self.set_state(|old_state| state_updater(old_state, result));
                return;
            }
            
            sleep(delay).await;
        }

        // 获取预设的结果
        let result = self.next_result::<T>();

        // 记录执行操作
        let operation = StateOperation::Execute {
            result: result.clone(),
        };
        self.record_operation(operation);

        // 更新状态
        self.set_state(|old_state| state_updater(old_state, result));
    }
}

/// 测试断言辅助函数
pub mod assert {
    use super::*;

    /// 断言状态是否符合预期
    pub fn assert_state<S: State + PartialEq + Debug>(store: &MockStateStore<S>, expected: S) {
        let state = store.get_state();
        assert_eq!(state, expected, "状态不符合预期");
    }

    /// 断言操作历史中是否包含特定操作
    pub fn assert_operation_contains<S: State + PartialEq, T: Clone + PartialEq + Send + 'static>(
        store: &MockStateStore<S>,
        expected: StateOperation<S, T>,
    ) {
        let operations = store.get_operations::<T>();
        assert!(
            operations.contains(&expected),
            "操作历史中不包含预期的操作"
        );
    }

    /// 断言操作历史中的操作数量
    pub fn assert_operation_count<S: State + PartialEq, T: Clone + PartialEq + Send + 'static>(
        store: &MockStateStore<S>,
        expected: usize,
    ) {
        let operations = store.get_operations::<T>();
        // 打印实际操作数量，帮助调试
        println!("预期操作数量: {}, 实际操作数量: {}", expected, operations.len());
        assert_eq!(
            operations.len(),
            expected,
            "操作历史中的操作数量不符合预期"
        );
    }

    /// 断言状态历史中包含特定状态
    pub fn assert_state_history_contains<S: State + PartialEq + Debug>(
        store: &MockStateStore<S>,
        expected: S,
    ) {
        let operations = store.operations.lock().unwrap();
        let contains = operations.iter().any(|op| {
            if let Some(state_op) = op.downcast_ref::<StateOperation<S, ()>>() {
                match state_op {
                    StateOperation::StateUpdate { new_state, .. } => new_state == &expected,
                    _ => false,
                }
            } else {
                false
            }
        });
        assert!(
            contains,
            "状态历史中不包含期望的状态: {:?}",
            expected
        );
    }

    /// 断言操作序列
    pub fn assert_operation_sequence<S: State + PartialEq + Debug, T: Clone + PartialEq + Send + Debug + 'static>(
        store: &MockStateStore<S>,
        expected: Vec<StateOperation<S, T>>,
    ) {
        let operations = store.get_operations::<T>();
        assert_eq!(
            operations.len(),
            expected.len(),
            "操作序列长度不匹配，期望{}个操作，实际{}个操作",
            expected.len(),
            operations.len()
        );

        for (i, (actual, expected)) in operations.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                actual, expected,
                "第{}个操作不匹配，\n期望: {:?},\n实际: {:?}",
                i, expected, actual
            );
        }
    }

    /// 断言状态转换
    pub fn assert_state_transition<S: State + PartialEq + Debug>(
        store: &MockStateStore<S>,
        from: S,
        to: S,
    ) {
        let operations = store.operations.lock().unwrap();
        let transition_exists = operations.iter().any(|op| {
            if let Some(state_op) = op.downcast_ref::<StateOperation<S, ()>>() {
                match state_op {
                    StateOperation::StateUpdate { old_state, new_state } => {
                        old_state == &from && new_state == &to
                    }
                    _ => false,
                }
            } else {
                false
            }
        });
        assert!(
            transition_exists,
            "未找到从 {:?} 到 {:?} 的状态转换",
            from,
            to
        );
    }

    /// 断言执行结果
    pub fn assert_execution_result<S: State + PartialEq, T: Clone + PartialEq + Send + Debug + 'static>(
        store: &MockStateStore<S>,
        expected_result: Async<T>,
    ) {
        let operations = store.get_operations::<T>();
        let result_exists = operations.iter().any(|op| match op {
            StateOperation::Execute { result } => result == &expected_result,
            _ => false,
        });
        assert!(
            result_exists,
            "未找到期望的执行结果: {:?}",
            expected_result
        );
    }
}

/// 用于创建模拟网络请求的工具
pub mod network {
    use super::*;
    use std::collections::{HashMap, VecDeque};

    /// 模拟的HTTP响应
    #[derive(Debug, Clone)]
    pub struct MockHttpResponse {
        pub status: u16,
        pub headers: HashMap<String, String>,
        pub body: Vec<u8>,
    }

    /// 模拟的HTTP请求
    #[derive(Debug, Clone)]
    pub struct MockHttpRequest {
        pub url: String,
        pub method: String,
        pub headers: HashMap<String, String>,
        pub body: Vec<u8>,
    }

    impl MockHttpRequest {
        /// 创建新的HTTP请求
        pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
            Self {
                method: method.into(),
                url: url.into(),
                headers: HashMap::new(),
                body: Vec::new(),
            }
        }

        /// 添加请求头
        pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
            self.headers.insert(name.into(), value.into());
            self
        }

        /// 设置请求体
        pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
            self.body = body.into();
            self
        }
    }

    impl MockHttpResponse {
        /// 创建新的HTTP响应
        pub fn new(status: u16, body: impl Into<Vec<u8>>) -> Self {
            Self {
                status,
                headers: HashMap::new(),
                body: body.into(),
            }
        }

        /// 添加响应头
        pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
            self.headers.insert(name.into(), value.into());
            self
        }

        /// 创建JSON响应
        pub fn json<T: Into<Vec<u8>>>(status: u16, body: T) -> Self {
            Self::new(status, body)
                .with_header("Content-Type", "application/json")
        }

        /// 创建文本响应
        pub fn text<T: Into<Vec<u8>>>(status: u16, body: T) -> Self {
            Self::new(status, body)
                .with_header("Content-Type", "text/plain")
        }
    }

    /// 请求验证器
    pub type RequestValidator = Box<dyn Fn(&MockHttpRequest) -> bool + Send + Sync>;

    /// 模拟的HTTP客户端
    pub struct MockHttpClient {
        responses: HashMap<String, VecDeque<MockHttpResponse>>,
        conditional_responses: Vec<(RequestValidator, MockHttpResponse)>,
        request_history: Vec<MockHttpRequest>,
    }

    impl MockHttpClient {
        /// 创建新的模拟HTTP客户端
        pub fn new() -> Self {
            Self {
                responses: HashMap::new(),
                conditional_responses: Vec::new(),
                request_history: Vec::new(),
            }
        }

        /// 为特定URL预设响应
        pub fn mock_response(&mut self, url: impl Into<String>, response: MockHttpResponse) {
            let url = url.into();
            let responses = self.responses.entry(url).or_insert_with(VecDeque::new);
            responses.push_back(response);
        }

        /// 为满足特定条件的请求预设响应
        pub fn mock_conditional_response<F>(&mut self, validator: F, response: MockHttpResponse)
        where
            F: Fn(&MockHttpRequest) -> bool + Send + Sync + 'static,
        {
            self.conditional_responses.push((Box::new(validator), response));
        }

        /// 发送GET请求
        pub async fn get(&mut self, url: &str) -> Result<MockHttpResponse, String> {
            let request = MockHttpRequest::new("GET", url);
            self.request(request).await
        }

        /// 发送POST请求
        pub async fn post(&mut self, url: &str, body: Vec<u8>) -> Result<MockHttpResponse, String> {
            let request = MockHttpRequest::new("POST", url).with_body(body);
            self.request(request).await
        }

        /// 发送PUT请求
        pub async fn put(&mut self, url: &str, body: Vec<u8>) -> Result<MockHttpResponse, String> {
            let request = MockHttpRequest::new("PUT", url).with_body(body);
            self.request(request).await
        }

        /// 发送DELETE请求
        pub async fn delete(&mut self, url: &str) -> Result<MockHttpResponse, String> {
            let request = MockHttpRequest::new("DELETE", url);
            self.request(request).await
        }

        /// 发送PATCH请求
        pub async fn patch(&mut self, url: &str, body: Vec<u8>) -> Result<MockHttpResponse, String> {
            let request = MockHttpRequest::new("PATCH", url).with_body(body);
            self.request(request).await
        }

        /// 发送HEAD请求
        pub async fn head(&mut self, url: &str) -> Result<MockHttpResponse, String> {
            let request = MockHttpRequest::new("HEAD", url);
            self.request(request).await
        }

        /// 发送OPTIONS请求
        pub async fn options(&mut self, url: &str) -> Result<MockHttpResponse, String> {
            let request = MockHttpRequest::new("OPTIONS", url);
            self.request(request).await
        }

        /// 获取请求历史
        pub fn get_request_history(&self) -> &[MockHttpRequest] {
            &self.request_history
        }

        /// 清除请求历史
        pub fn clear_request_history(&mut self) {
            self.request_history.clear();
        }

        /// 处理请求
        async fn request(&mut self, request: MockHttpRequest) -> Result<MockHttpResponse, String> {
            // 记录请求
            self.request_history.push(request.clone());

            // 检查条件响应
            for (validator, response) in &self.conditional_responses {
                if validator(&request) {
                    return Ok(response.clone());
                }
            }

            // 检查URL响应
            if let Some(responses) = self.responses.get_mut(&request.url) {
                if let Some(response) = responses.pop_front() {
                    return Ok(response);
                }
            }

            Err(format!("没有为URL '{}' 预设响应", request.url))
        }
    }

    /// 断言辅助函数
    pub mod assert {
        use super::*;

        /// 断言请求历史中包含特定URL的请求
        pub fn assert_requested_url(client: &MockHttpClient, url: &str) {
            let history = client.get_request_history();
            assert!(
                history.iter().any(|req| req.url == url),
                "没有找到对URL '{}'的请求",
                url
            );
        }

        /// 断言请求历史中包含特定方法的请求
        pub fn assert_requested_method(client: &MockHttpClient, method: &str, url: &str) {
            let history = client.get_request_history();
            assert!(
                history.iter().any(|req| req.method == method && req.url == url),
                "没有找到对URL '{}'的{}请求",
                url,
                method
            );
        }

        /// 断言请求历史中包含特定请求体的请求
        pub fn assert_requested_body(client: &MockHttpClient, url: &str, body_contains: &[u8]) {
            let history = client.get_request_history();
            assert!(
                history.iter().any(|req| req.url == url && req.body.windows(body_contains.len()).any(|window| window == body_contains)),
                "没有找到对URL '{}'且请求体包含指定内容的请求",
                url
            );
        }

        /// 断言请求历史中包含特定请求头的请求
        pub fn assert_requested_header(client: &MockHttpClient, url: &str, header: &str, value: &str) {
            let history = client.get_request_history();
            assert!(
                history.iter().any(|req| req.url == url && req.headers.get(header).map_or(false, |v| v == value)),
                "没有找到对URL '{}'且请求头'{}'为'{}'的请求",
                url,
                header,
                value
            );
        }

        /// 断言请求次数
        pub fn assert_request_count(client: &MockHttpClient, expected: usize) {
            let count = client.get_request_history().len();
            assert_eq!(count, expected, "请求次数不匹配，期望{}次，实际{}次", expected, count);
        }
    }
}

/// 模拟事件流
pub mod event_stream {
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};
    use std::time::Duration;

    use futures_core::stream::Stream;
    use tokio::time::{sleep, Sleep};

    /// 模拟事件流
    pub struct MockEventStream<T> {
        events: Arc<Mutex<Vec<(T, Option<Duration>)>>>,
        current_index: Arc<Mutex<usize>>,
        sleep: Arc<Mutex<Option<Pin<Box<Sleep>>>>>,
    }

    impl<T: Clone + Send + 'static> MockEventStream<T> {
        /// 创建新的模拟事件流
        pub fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
                current_index: Arc::new(Mutex::new(0)),
                sleep: Arc::new(Mutex::new(None)),
            }
        }

        /// 添加事件
        pub fn add_event(&self, event: T) {
            let mut events = self.events.lock().unwrap();
            events.push((event, None));
        }

        /// 添加带延迟的事件
        pub fn add_delayed_event(&self, event: T, delay: Duration) {
            let mut events = self.events.lock().unwrap();
            events.push((event, Some(delay)));
        }

        /// 添加多个事件
        pub fn add_events(&self, events: Vec<T>) {
            let mut current_events = self.events.lock().unwrap();
            for event in events {
                current_events.push((event, None));
            }
        }

        /// 添加多个带延迟的事件
        pub fn add_delayed_events(&self, events: Vec<(T, Duration)>) {
            let mut current_events = self.events.lock().unwrap();
            for (event, delay) in events {
                current_events.push((event, Some(delay)));
            }
        }

        /// 清除所有事件
        pub fn clear(&self) {
            let mut events = self.events.lock().unwrap();
            events.clear();
            *self.current_index.lock().unwrap() = 0;
            *self.sleep.lock().unwrap() = None;
        }
    }

    impl<T: Clone + Send + 'static> Stream for MockEventStream<T> {
        type Item = T;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            // 如果有待处理的延迟，先处理它
            let mut sleep_guard = self.sleep.lock().unwrap();
            if let Some(sleep) = sleep_guard.as_mut() {
                match sleep.as_mut().poll(cx) {
                    Poll::Ready(_) => {
                        *sleep_guard = None;
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }
            drop(sleep_guard);

            let events = self.events.lock().unwrap();
            let mut current_index = self.current_index.lock().unwrap();
            
            // 检查是否还有事件
            if *current_index >= events.len() {
                return Poll::Ready(None);
            }

            let (event, delay) = &events[*current_index];
            *current_index += 1;
            drop(current_index);

            // 如果有延迟，设置睡眠时间
            if let Some(delay) = delay {
                let mut sleep_guard = self.sleep.lock().unwrap();
                *sleep_guard = Some(Box::pin(sleep(*delay)));
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }

            Poll::Ready(Some(event.clone()))
        }
    }

    /// 创建模拟事件流的辅助函数
    pub fn mock_stream<T: Clone + Send + 'static>(events: Vec<T>) -> MockEventStream<T> {
        let stream = MockEventStream::new();
        stream.add_events(events);
        stream
    }

    /// 创建带延迟的模拟事件流的辅助函数
    pub fn mock_delayed_stream<T: Clone + Send + 'static>(events: Vec<(T, Duration)>) -> MockEventStream<T> {
        let stream = MockEventStream::new();
        stream.add_delayed_events(events);
        stream
    }
}