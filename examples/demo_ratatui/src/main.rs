use main_runner::MainRunner;
use crate::tracing_setup::tracing_init;
use crossterm::ExecutableCommand;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::stdout;
use std::sync::Arc;

mod counter;
mod executor;
mod input;
mod progress;

mod tracing_setup;
pub mod main_runner;
pub mod main_view;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_init();
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    
    let result = MainRunner::new().run(terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result
}
