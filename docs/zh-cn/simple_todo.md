# EaseRx 框架 Todo 应用示例分析

本文档将深入分析一个基于 Rust 语言和 `EaseRx` 响应式 MVI 框架构建的命令行（CLI）Todo 应用。我们将详细剖析其项目架构、核心组件和实现逻辑，旨在展示 `EaseRx` 如何简化 Rust 中的异步编程和状态管理。

使用`just todo`来运行此示例

## 1\. 项目概述

此项目是一个功能完备的 Todo 应用，它利用 `tokio` 作为异步运行时，并围绕 `EaseRx` 框架的核心理念构建。应用在启动后会自动模拟一系列操作，如添加、完成和清除待办事项，并通过响应式的数据流实时更新控制台界面。

该示例清晰地展示了 `EaseRx` 框架如何实现其核心目标：

* **响应式状态管理**：应用状态的任何变更都会自动触发 UI 的重新渲染。
* **简化的异步处理**：将复杂的异步操作（如模拟耗时计算）封装为简洁的、类似同步代码的调用方式。
* **清晰的架构分离**：通过 MVI（Model-View-Intent）模式，明确划分了状态、业务逻辑和视图的职责。

## 2\. 架构设计：MVI 模式

本项目采用了 MVI（Model-View-Intent）架构模式，确保了单向数据流和清晰的职责分离。

* **Model（模型）**：负责管理应用唯一且集中的状态（State），并包含所有用于操作该状态的业务逻辑。

    * State (`todo_state.rs`)：定义了应用的所有数据结构，是应用状态的“唯一事实来源”（Single Source of Truth）。
    * Model (`todo_model.rs`)：封装了 `StateStore`，并对外提供修改状态的接口（例如 `add_todo`）。

* **View（视图）**：负责渲染用户界面。在本例中，它是一个简单的函数，根据当前状态将 Todo 列表渲染到控制台。

    * View (`todo_view.rs`)：它是一个无状态的组件，其渲染结果完全由输入的当前状态决定。

* **Intent（意图）与响应式循环**：用户的操作或应用内的事件被视作“意图”。在本项目中，`main.rs` 文件中的主循环和异步任务模拟了这些意图，并驱动状态变更。响应式循环则监听状态变更，并自动调用视图进行更新。

## 3\. 核心组件分析

下面我们详细分析构成应用的关键模块。

### 3.1. `todo_state.rs`：定义应用状态

此文件定义了应用的所有数据结构。

* `Todo`: 一个简单的结构体，包含 `text` 和 `completed` 字段，代表一个待办事项。
* `TodoState`: 这是整个应用的核心状态。
  ```rust
  #[derive(Debug, Clone, Default)]
  pub struct TodoState {
      pub todos: Arc<Mutex<Vec<Todo>>>,
      pub play: Async<u64>,
      pub exit: bool,
  }
  ```
    * `todos`: 使用 `Arc<Mutex<>>` 包装的待办事项列表，允许多线程安全访问。
    * `play`: 类型为 `Async<u64>`。这是 `EaseRx` 框架的核心类型，用于封装异步操作的生命周期状态，包括：`Uninitialized`（未初始化）、`Loading`（加载中）、`Success`（成功）和 `Fail`（失败）。
    * `exit`: 一个布尔标志，用于控制应用主循环的退出。
* **不可变状态更新**：`TodoState` 的所有修改函数（如 `add_todo`, `remove_completed_todos`）都遵循函数式编程的理念。它们接收当前状态的所有权（`self`），然后返回一个全新的、修改后的状态实例，从而保证了状态更新的可预测性。

### 3.2. `todo_model.rs`：封装业务逻辑

`TodoModel` 是业务逻辑的中心，它通过持有一个 `StateStore` 来间接管理 `TodoState`。外部代码通过 `TodoModel` 提供的方法来触发状态变更。

* **`StateStore`**: 这是 `EaseRx` 框架提供的核心状态容器。`TodoModel` 使用它来原子化地更新和分发状态。

* **同步状态变更**: 对于简单的操作，如添加待办事项，`TodoModel` 直接调用 `store.set_state`。

  ```rust
  pub fn add_todo(&self, text: &str) -> Result<(), AsyncError> {
      let todo = Todo::new(text);
      self.store.set_state(|state| state.add_todo(todo))
  }
  ```

* **异步操作封装**: `resolve_todo` 方法是 `EaseRx` 框架强大功能的绝佳示例。

  ```rust
  pub fn resolve_todo(&self, index: usize) -> JoinHandle<Result<(), AsyncError>> {
      self.store.execute(
          || fibonacci_result(92), // 一个可能耗时且会失败的函数
          move |state, num| state.resolve_todo(index, num),
      )
  }
  ```

  这里，`store.execute` 方法处理了异步操作的所有复杂细节：

    1.  它会自动将 `fibonacci_result` 的执行状态（执行中、成功或失败）封装进 `Async<T>` 类型。
    2.  然后，它将这个 `Async` 对象传递给状态更新闭包。
    3.  开发者只需在 `state.resolve_todo` 中定义如何根据 `Async` 的不同状态（如 `is_success()`）来更新 `TodoState` 即可。

  **注意**：这种模式实现了EaseRx的核心需求：“以同步代码的写法编写异步代码”，极大地降低了异步编程的心智负担。

### 3.3. `todo_view.rs`：纯粹的视图渲染

`show_todos` 函数是本应用的“视图”。它是一个纯函数，职责非常单一：

* 接收当前的应用状态（`todos`, `progress`, `play`）作为输入。
* 根据输入的状态，清空屏幕并打印出最新的待办事项列表和进度。
* 它不包含任何业务逻辑，也不直接修改任何状态。

### 3.4. `main.rs`：应用入口与响应式循环

`main.rs` 负责组装并启动整个应用。

* **初始化**: 创建 `TodoModel` 实例。
* **模拟用户意图**: `tokio::task::spawn` 中启动了一个异步任务，它按顺序、带有时延地调用 `TodoModel` 的各种方法，模拟了一个完整的使用流程。
  ```rust
  tokio::task::spawn(async move {
      model.add_todo("Build a Todo App")?;
      sleep(Duration::from_secs(1)).await;
      model.set_todo_completed(0, true)?;
      // ... 其他操作
      model.exit()?; // 触发退出
      Ok::<(), AsyncError>(())
  });
  ```
* **响应式循环**: `main` 函数的核心是 `futures_signals` 驱动的响应式循环。
  ```rust
  model_clone
      .store()
      .to_signal() // 1. 从 StateStore 创建一个状态信号（Signal）
      .stop_if(|state| state.exit) // 2. 当 state.exit 为 true 时，自动停止监听
      .for_each(|state| { // 3. 订阅状态变更
          show_todos(state.todos.clone(), state.todo_progress(), state.play);
          async {}
      })
      .await;
  ```
  这段代码构建了一个数据管道：每当 `StateStore` 中的状态发生变化，`to_signal` 就会发出一个新的状态值，`for_each` 会接收到这个新状态，并立即调用 `show_todos` 函数来更新控制台的显示。这套机制是“响应式编程”的经典体现。

## 4\. 总结

这个 Todo 应用虽然简单，但它完美地诠释了 `EaseRx` 框架的设计哲学和核心优势。通过 MVI 架构实现了清晰的职责划分，利用 `StateStore` 和 `Async<T>` 等核心组件，将原本复杂的异步数据流和状态管理过程变得井然有序、易于理解和维护。开发者可以更加专注于业务逻辑本身，而将状态同步和异步处理的公共难题交给框架解决。