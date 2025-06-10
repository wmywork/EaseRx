use crate::todo::todo_state::{Todo, TodoState};
use easerx::{AsyncError, StateStore};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::task::JoinHandle;

pub struct TodoModel {
    store: Arc<StateStore<TodoState>>,
}

impl TodoModel {
    pub fn new() -> Self {
        Self {
            store: Arc::new(StateStore::new(TodoState::default())),
        }
    }

    pub fn store(&self) -> Arc<StateStore<TodoState>> {
        self.store.clone()
    }

    pub fn exit(&self) -> Result<(), AsyncError> {
        self.store.set_state(|state| state.set_exit())
    }

    pub fn add_todo(&self, text: &str) -> Result<(), AsyncError> {
        let todo = Todo::new(text);
        self.store.set_state(|state| state.add_todo(todo))
    }

    pub fn remove_completed_todos(&self) -> Result<(), AsyncError> {
        self.store.set_state(move |state| state.remove_completed_todos())
    }

    pub fn resolve_todo(&self, index: usize) -> JoinHandle<Result<(), AsyncError>> {
        self.store.execute(
            || fibonacci_result(92),
            move |state, num| state.resolve_todo(index, num),
        )
    }

    pub fn set_todo_completed(&self, index: usize, completed: bool) -> Result<(), AsyncError> {
        self.store
            .set_state(move |state| state.set_todo_completed(index, completed))
    }
}

fn fibonacci_result(n: u64) -> Result<u64, String> {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        match a.checked_add(b) {
            Some(next) => {
                a = b;
                b = next;
            }
            None => return Err(format!("calculation overflow at n={}", n)),
        }
    }
    sleep(Duration::from_secs(1));
    Ok(a)
}
