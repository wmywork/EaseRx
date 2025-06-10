use crate::app::app_view::app_view;
use crate::counter::counter_model::CounterViewModel;
use crate::executor::executor_model::ExecutorModel;
use crate::input::input_handler::InputHandler;
use crate::progress::progress_model::ProgressViewModel;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct AppModel {
    progress_model: Arc<ProgressViewModel>,
    counter_model: Arc<CounterViewModel>,
    executor_model: Arc<ExecutorModel>,
    input_handler: Arc<InputHandler>,
}

impl AppModel {
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

    pub async fn run(&self, mut terminal: Terminal<CrosstermBackend<Stdout>>) {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100);

        loop {
            let progress_state = (*self.progress_model.store()).get_state();
            let counter_state = (*self.counter_model.store()).get_state();
            let executor_state = (*self.executor_model.store()).get_state();
            let input_state = (*self.input_handler.store).get_state();

            if input_state.exit {
                break;
            }

            let result = terminal.draw(|frame| {
                app_view(frame, &progress_state, &counter_state, &executor_state);
            });

            if result.is_err() {
                tracing::error!("Failed to draw the terminal: {:?}", result.err());
                self.input_handler.request_exit();
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap_or(false) {
                if let Ok(event) = event::read() {
                    self.handle_event(event);
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if (*self.counter_model.store()).get_state().started {
                    self.counter_model
                        .store()
                        ._set_state(|s| s.on_counter_increment());
                }
                if (*self.executor_model.store()).get_state().async_num.is_loading() {
                    self.executor_model.store()._set_state(|s| s.on_tick());
                }
                last_tick = Instant::now();
            }
        }
    }

    fn handle_event(&self, event: Event) {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return;
            }

            // Ctrl-C to exit
            if let KeyCode::Char('c') = key.code {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.input_handler.request_exit();
                    return;
                }
            }

            match key.code {
                //App
                KeyCode::Esc => self.input_handler.request_exit(),
                KeyCode::Char('r') => {
                    self.progress_model.reset_progress();
                    self.counter_model.reset_counter()
                }
                //Progress
                KeyCode::Up => self.progress_model.change_color_up(),
                KeyCode::Down => self.progress_model.change_color_down(),
                KeyCode::Left => self.progress_model.decrement_progress(),
                KeyCode::Right => {
                    self.progress_model.increment_progress();
                }
                //Counter
                KeyCode::Char('-') | KeyCode::Char('_') => self.counter_model.decrement_counter(),
                KeyCode::Char('+') | KeyCode::Char('=') => self.counter_model.increment_counter(),
                KeyCode::Char('o') => self.counter_model.start_counter(),
                KeyCode::Char('p') => self.counter_model.stop_counter(),
                //Executor
                KeyCode::Char('c') => self.executor_model.request_calc(),
                _ => {}
            }
        }
    }
}
