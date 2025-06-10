use crate::todo::todo_model::TodoModel;
use crate::todo::todo_view::show_todos;
use crate::tracing_setup::tracing_init;
use easerx::AsyncError;
use futures_signals::signal::SignalExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

mod todo;
mod tracing_setup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init();

    //Create model
    let model = Arc::new(TodoModel::new());
    let model_clone = model.clone();

    tokio::task::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        model.add_todo("Build a Todo App")?;
        sleep(Duration::from_secs(1)).await;
        model.add_todo("Contribute to Open Source")?;
        sleep(Duration::from_secs(1)).await;
        model.add_todo("Read Rust Book")?;
        sleep(Duration::from_secs(1)).await;
        model.add_todo("Learn Async Rust")?;
        sleep(Duration::from_secs(1)).await;
        model.add_todo("Play a game")?;
        sleep(Duration::from_secs(1)).await;
        model.add_todo("Have breakfast")?;

        sleep(Duration::from_secs(1)).await;
        model.set_todo_completed(0, true)?;
        sleep(Duration::from_secs(1)).await;
        model.set_todo_completed(1, true)?;
        sleep(Duration::from_secs(1)).await;
        model.set_todo_completed(3, true)?;
        sleep(Duration::from_secs(1)).await;
        model.resolve_todo(4);
        sleep(Duration::from_secs(3)).await;
        model.remove_completed_todos()?;
        sleep(Duration::from_secs(2)).await;

        model.exit()?;
        Ok::<(), AsyncError>(())
    });

    model_clone
        .store()
        .to_signal()
        .stop_if(|state| state.exit)
        .for_each(|state| {
            show_todos(state.todos.clone(), state.todo_progress(), state.play);
            async {}
        })
        .await;
    info!("=================================");
    info!("  Main thread | Finish");
    Ok(())
}
