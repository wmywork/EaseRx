# EaseRx - Rust响应式MVI框架

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

EaseRx是一个Rust语言的响应式MVI (Model-View-Intent) 框架，旨在降低Rust异步编程的心智负担，提供简洁直观的状态管理方案。

## 项目概述

Rust的学习曲线较陡峭，尤其是在处理异步编程和状态管理方面。EaseRx通过提供结构化的架构，让开发者能够以同步代码的写法编写异步代码，并自动完成错误转换，从而简化这一过程。

## 核心功能

- **响应式状态管理**：提供完整的响应式状态管理解决方案，支持状态变更的自动传播
- **简化异步编程**：以同步代码的写法编写异步代码，隐藏异步编程的复杂性
- **自动错误转换**：统一错误处理机制，自动将各种错误类型转换为框架内的统一表示
- **可取消操作**：支持长时间运行操作的取消
- **超时处理**：内置操作超时支持
- **值保留**：在加载新数据时保留之前的值

## 主要组件

### 状态存储 (StateStore)

`StateStore<S>`是EaseRx的核心组件，负责管理应用状态，提供响应式的状态变更接口：

```
let store = StateStore::new(AppState { counter: 0 });

// 更新状态
store.set_state(|state| {
    state.add_count(1)
});

// 执行一个更新状态的操作
store.execute(
    || computation(),
    | state, result| {
        sate.set_data(result)
    }
);
```

### 异步状态表示 (Async)

`Async<T>`封装异步操作的各种状态：

- `Uninitialized`：未初始化状态
- `Loading`：加载中状态，可选保留之前的值
- `Success`：成功状态，包含结果值
- `Fail`：失败状态，包含错误信息和可选的之前值

```
match state.data {
    Async::Uninitialized => println!("未开始"),
    Async::Loading(_) => println!("加载中..."),
    Async::Success { value } => println!("数据: {}", value),
    Async::Fail { error, .. } => println!("错误: {}", error),
}
```

## 执行方法

EaseRx提供多种执行方法以适应不同场景：

- **同步操作**:
  - `execute`：基本同步操作
  - `execute_with_retain`：加载过程中保留之前的值
  - `execute_cancellable`：支持取消操作
  - `execute_with_timeout`：自动超时处理

- **异步操作**:
  - `async_execute`：基本异步操作
  - `async_execute_with_retain`：加载过程中保留之前的值
  - `async_execute_cancellable`：支持取消操作
  - `async_execute_with_timeout`：自动超时处理

## 使用场景

- **UI状态管理**：在GUI应用中管理组件状态
- **网络请求处理**：处理加载状态、成功响应和错误情况
- **数据处理流水线**：构建数据转换、过滤和聚合操作

## 快速开始

在`Cargo.toml`中添加EaseRx依赖：

```toml
[dependencies]
easerx = "0.1.0"
```

基本使用示例：

```rust
use easerx::{StateStore, State, Async};
use std::time::Duration;

#[derive(Clone)]
struct AppState {
    data: Async<String>,
}

impl State for AppState {}

fn main() {
    // 创建一个新的状态存储
    let store = StateStore::new(AppState { 
        data: Async::Uninitialized 
    });
    
    // 执行一个带超时的异步操作
    store.async_execute_with_timeout(
        async {
            // 模拟网络请求
            tokio::time::sleep(Duration::from_millis(100)).await;
            "数据加载成功".to_string()
        },
        Duration::from_secs(1),
        |mut state, result| {
            state.data = result;
            state
        }
    );
    
    // 订阅状态变更
    let stream = store.to_stream();
    // 处理状态更新...
}
```

## 技术依赖

- [tokio](https://github.com/tokio-rs/tokio)：异步运行时
- [futures-signals](https://github.com/Pauan/rust-signals)：响应式编程支持
- [thiserror](https://github.com/dtolnay/thiserror)：错误处理
- [pin-project](https://github.com/taiki-e/pin-project)：Pin投影

## 设计原则

1. **简单性**：API设计简洁明了，易于理解和使用
2. **一致性**：保持一致的接口和行为模式
3. **可组合性**：组件可以灵活组合以满足不同需求
4. **类型安全**：充分利用Rust的类型系统保障代码安全
5. **性能优先**：在设计决策中优先考虑性能影响

## 许可证

本项目采用MIT许可证 - 详情请参阅LICENSE文件 