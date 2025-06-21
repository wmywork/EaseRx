use crate::counter::counter_model::CounterViewModel;
use crate::executor::executor_model::ExecutorModel;
use crate::input::input_handler::{start_input_listener, InputHandler};
use crate::main_view::app_view;
use crate::progress::progress_model::ProgressViewModel;
use easerx::combine_state_flow;
use futures::StreamExt;
use futures_signals::map_ref;
use futures_signals::signal::SignalExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::sync::Arc;

pub struct MainRunner {
    progress_model: Arc<ProgressViewModel>,
    counter_model: Arc<CounterViewModel>,
    executor_model: Arc<ExecutorModel>,
    input_handler: Arc<InputHandler>,
}

impl Default for MainRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MainRunner {
    pub fn new() -> Self {
        let progress_model = ProgressViewModel::new();
        let counter_model = CounterViewModel::new();
        let executor_model = ExecutorModel::new();
        let input_handler = InputHandler::new();
        Self {
            progress_model: Arc::new(progress_model),
            counter_model: Arc::new(counter_model),
            executor_model: Arc::new(executor_model),
            input_handler: Arc::new(input_handler),
        }
    }

    pub async fn run(
        &self,
        mut terminal: Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let progress_model = self.progress_model.clone();
        let counter_model = self.counter_model.clone();
        let executor_model = self.executor_model.clone();
        let input_handler = self.input_handler.clone();
        start_input_listener(
            progress_model.clone(),
            counter_model.clone(),
            executor_model.clone(),
            input_handler.clone(),
        );

        let mut state_flow = combine_state_flow! {
            progress_model.store().to_signal(),
            counter_model.store().to_signal(),
            executor_model.store().to_signal(),
            input_handler.store().to_signal(),
        }
        .to_stream();

        while let Some((progress_state, counter_state, executor_state, input_state)) =
            state_flow.next().await
        {
            if input_state.exit {
                self.counter_model.request_exit();
                self.executor_model.request_exit();
                break;
            }

            terminal.draw(|frame| {
                app_view(frame, &progress_state, &counter_state, &executor_state);
            })?;
        }

        Ok(())
    }
}
