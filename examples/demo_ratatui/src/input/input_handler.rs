use crate::counter::counter_model::CounterViewModel;
use crate::executor::executor_model::ExecutorModel;
use crate::input::input_state::InputState;
use crate::progress::progress_model::ProgressViewModel;
use crate::Arc;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use easerx::StateStore;
use std::time::Duration;
use tracing::error;

pub struct InputHandler {
    pub store: Arc<StateStore<InputState>>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            store: Arc::new(StateStore::new(InputState::default())),
        }
    }

    pub fn store(&self) -> Arc<StateStore<InputState>> {
        self.store.clone()
    }

    pub fn send_draw_event(&self) {
        self.store._set_state(|state| state.send_draw_event());
    }

    pub fn request_exit(&self) {
        self.store._set_state(|state| state.set_exit());
    }
}

pub fn start_input_listener(
    progress_model: Arc<ProgressViewModel>,
    counter_model: Arc<CounterViewModel>,
    executor_model: Arc<ExecutorModel>,
    input_handler: Arc<InputHandler>,
) {
    let timeout = Duration::from_millis(10);
    let input_store = input_handler.store.clone();
    tokio::spawn(async move {
        loop {
            let input_state = input_store.await_state().await;
            match input_state {
                Err(_) => {
                    error!("Failed to get input state");
                    break;
                }
                Ok(state) => {
                    if state.is_exit() {
                        break;
                    }
                }
            }
            if event::poll(timeout).unwrap_or(false) {
                if let Ok(event) = event::read() {
                    handle_event(
                        event,
                        progress_model.clone(),
                        counter_model.clone(),
                        executor_model.clone(),
                        input_handler.clone(),
                    );
                }
            }
        }
    });
}

fn handle_event(
    event: Event,
    progress_model: Arc<ProgressViewModel>,
    counter_model: Arc<CounterViewModel>,
    executor_model: Arc<ExecutorModel>,
    input_handler: Arc<InputHandler>,
) {
    if let Event::Resize(_, _) = event {
        input_handler.send_draw_event()
    }

    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Ctrl-C to exit
        if let KeyCode::Char('c') = key.code {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                input_handler.request_exit();
                return;
            }
        }

        match key.code {
            //App
            KeyCode::Esc => input_handler.request_exit(),
            KeyCode::Char('r') => {
                progress_model.reset_progress();
                counter_model.reset_counter();
                executor_model.reset_num();
            }
            //Progress
            KeyCode::Up => progress_model.change_color_up(),
            KeyCode::Down => progress_model.change_color_down(),
            KeyCode::Left => progress_model.decrement_progress(),
            KeyCode::Right => {
                progress_model.increment_progress();
            }
            //Counter
            KeyCode::Char('-') | KeyCode::Char('_') => counter_model.decrement_count(),
            KeyCode::Char('+') | KeyCode::Char('=') => counter_model.increment_count(),
            KeyCode::Char('o') => counter_model.start_counter(),
            KeyCode::Char('p') => counter_model.stop_counter(),
            //Executor
            KeyCode::Char('c') => executor_model.request_calc(),
            _ => {}
        }
    }
}
