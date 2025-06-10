use crate::app::app_view::app_view;
use crate::counter::counter_model::CounterViewModel;
use crate::executor::executor_model::ExecutorModel;
use crate::input::input_handler::InputHandler;
use crate::progress::progress_model::ProgressViewModel;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use easerx::combine_state_flow;
use futures::StreamExt;
use futures_signals::map_ref;
use futures_signals::signal::SignalExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;
use std::sync::Arc;

pub struct AppModel {
    progress_model: Arc<ProgressViewModel>,
    counter_model: Arc<CounterViewModel>,
    executor_model: Arc<ExecutorModel>,
}

impl AppModel {
    pub fn new() -> Self {
        let progress_model = ProgressViewModel::new();
        let counter_model = CounterViewModel::new();
        let executor_model = ExecutorModel::new();
        Self {
            progress_model: Arc::new(progress_model),
            counter_model: Arc::new(counter_model),
            executor_model: Arc::new(executor_model),
        }
    }

    pub async fn run(&self, mut terminal: Terminal<CrosstermBackend<Stdout>>) {
        let (input_handler, input_rx) = InputHandler::new();
        let input_handler = Arc::new(input_handler);

        let input_handler_clone = input_handler.clone();
        let progress_model = self.progress_model.clone();
        let counter_model = self.counter_model.clone();
        let executor_model = self.executor_model.clone();

        InputHandler::start_dispatcher(input_rx, move |event| {
            if let Event::Key(key) = event {
                if let KeyCode::Char('c') = key.code {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        input_handler.request_exit();
                    }
                }
            }
            match event {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            //App
                            KeyCode::Esc => input_handler.request_exit(),
                            KeyCode::Char('r') => {
                                progress_model.reset_progress();
                                counter_model.reset_counter()
                            }
                            //Progress
                            KeyCode::Up => progress_model.change_color_up(),
                            KeyCode::Down => progress_model.change_color_down(),
                            KeyCode::Left => progress_model.decrement_progress(),
                            KeyCode::Right => {
                                progress_model.increment_progress();
                            }
                            //Counter
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                counter_model.decrement_counter()
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                counter_model.increment_counter()
                            }
                            KeyCode::Char('o') => counter_model.start_counter(),
                            KeyCode::Char('p') => counter_model.stop_counter(),
                            //Executor
                            KeyCode::Char('c') => executor_model.request_calc(),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        });

        let app_state = combine_state_flow! {
            self.progress_model.store().to_signal(),
            self.counter_model.store().to_signal(),
            self.executor_model.store().to_signal(),
            input_handler_clone.store.to_signal(),
        };

        app_state
            .stop_if(|(_, _, _, input_state)| input_state.exit)
            .to_stream()
            .for_each(|(progress_state, counter_state, executor_state, _)| {
                let result = terminal.draw(|frame| {
                    app_view(frame, &progress_state, &counter_state, &executor_state);
                });
                if result.is_err() {
                    tracing::error!("Failed to draw the terminal: {:?}", result.err());
                }
                async move {}
            })
            .await;
    }
}
