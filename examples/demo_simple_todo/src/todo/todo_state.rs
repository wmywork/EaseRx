use easerx::{Async, State};
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct Todo {
    pub text: String,
    pub completed: bool,
}

impl Todo {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            completed: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TodoState {
    pub todos: Arc<Mutex<Vec<Todo>>>,
    pub play: Async<u64>,
    pub exit: bool,
}

impl State for TodoState {}

impl TodoState {
    pub fn add_todo(self, value: Todo) -> Self {
        self.todos.lock().unwrap().push(value);
        self
    }

    pub fn remove_completed_todos(self) -> Self {
        {
            let mut todos = self.todos.lock().unwrap();
            todos.retain(|todo| !todo.completed);
        }
        self
    }

    pub fn set_todo_completed(self, index: usize, completed: bool) -> Self {
        {
            let mut todos = self.todos.lock().unwrap();
            if index < todos.len() {
                todos[index].completed = completed;
            }
        }
        self
    }

    pub fn resolve_todo(mut self, index: usize, ponder: Async<u64>) -> Self {
        {
            let mut todos = self.todos.lock().unwrap();
            if index < todos.len() {
                todos[index].completed = ponder.is_success();
            }
            self.play = ponder;
        }
        self
    }

    pub fn todos_count(&self) -> usize {
        self.todos.lock().unwrap().len()
    }

    pub fn todo_completed_count(&self) -> usize {
        self.todos
            .lock()
            .unwrap()
            .iter()
            .filter(|todo| todo.completed)
            .count()
    }

    pub fn todo_progress(&self) -> TodoProgress {
        let total = self.todos_count();
        if total == 0 {
            TodoProgress::default()
        } else {
            let completed = self.todo_completed_count();
            let percentage = (completed as f64 / total as f64) * 100.0;
            TodoProgress {
                completed,
                total,
                percentage,
            }
        }
    }

    pub fn set_exit(self) -> Self {
        Self { exit: true, ..self }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TodoProgress {
    pub completed: usize,
    pub total: usize,
    pub percentage: f64,
}

impl Display for TodoProgress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Progress:{}/{} Percentage:{:.2}%",
            self.completed, self.total, self.percentage
        )
    }
}
