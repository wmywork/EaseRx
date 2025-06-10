use crate::app::app_model::AppModel;
use crate::tracing_setup::tracing_init;
use crossterm::ExecutableCommand;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::stdout;
use std::sync::Arc;

mod app;
mod counter;
mod executor;
mod input;
mod progress;

mod tracing_setup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init();
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let main_view = Arc::new(AppModel::new());

    main_view.run(terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
