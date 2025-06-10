use crate::todo::todo_state::{Todo, TodoProgress};
use easerx::Async;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

fn clear_screen() {
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd").args(["/c", "cls"]).status();
    } else {
        let _ = Command::new("clear").status();
    }
}

pub fn show_todos(todos: Arc<Mutex<Vec<Todo>>>, progress: TodoProgress, play: Async<u64>) {
    let todos = todos.lock().unwrap();
    clear_screen();
    info!("=================================");
    debug!("| {progress}");
    debug!("| Play: {:?}", play);
    if todos.is_empty() {
        debug!("| No todos available.");
    } else {
        for (index, todo) in todos.iter().enumerate() {
            let status = if todo.completed { "✓" } else { " " };
            debug!("| [{}] {} {}", index, status, todo.text);
        }
    }
}
