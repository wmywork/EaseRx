use crate::counter::counter_model::CounterModel;
use crate::executor::executor_model::ExecutorModel;
use crate::input::input_state::InputState;
use crate::progress::progress_model::ProgressModel;
use cursive::event::Key::Esc;
use cursive::event::{Event, Key};
use cursive::Cursive;
use easerx::StateStore;
use std::sync::Arc;

pub struct InputHandler {
    pub store: Arc<StateStore<InputState>>,
}

impl InputHandler {
    // Create a new InputHandler with default state
    pub fn new() -> Self {
        Self {
            store: Arc::new(StateStore::new(InputState::default())),
        }
    }

    // Get a clone of the state store
    pub fn store(&self) -> Arc<StateStore<InputState>> {
        self.store.clone()
    }

    // Request application exit
    pub fn request_exit(&self) {
        self.store._set_state(|state| state.set_exit());
    }
}

pub fn setup_event_handler(
    siv: &mut Cursive,
    progress_model: Arc<ProgressModel>,
    counter_model: Arc<CounterModel>,
    executor_model: Arc<ExecutorModel>,
    input_handler: Arc<InputHandler>,
) {
    {
        let counter_model = counter_model.clone();
        let input_handler = input_handler.clone();
        siv.add_global_callback(Event::CtrlChar('c'), move |s| {
            input_handler.request_exit();
            counter_model.request_exit();
            s.quit();
        });
    }
    {
        let counter_model = counter_model.clone();
        let input_handler = input_handler.clone();
        siv.add_global_callback(Esc, move |s| {
            input_handler.request_exit();
            counter_model.request_exit();
            s.quit();
        });
    }
    {
        let counter_model = counter_model.clone();
        let progress_model = progress_model.clone();
        let executor_model = executor_model.clone();
        siv.add_global_callback('r', move |_| {
            counter_model.reset_counter();
            progress_model.reset_progress();
            executor_model.reset_num();
        });
    }

    // 处理方向键 - 上
    {
        let progress_model = progress_model.clone();
        siv.add_global_callback(Event::Key(Key::Up), move |_| {
            progress_model.change_color_up();
        });
    }

    // 处理方向键 - 下
    {
        let progress_model = progress_model.clone();
        siv.add_global_callback(Event::Key(Key::Down), move |_| {
            progress_model.change_color_down();
        });
    }

    // 处理方向键 - 左
    {
        let progress_model = progress_model.clone();
        siv.add_global_callback(Event::Key(Key::Left), move |_| {
            progress_model.decrement_progress();
        });
    }

    // 处理方向键 - 右
    {
        let progress_model = progress_model.clone();
        siv.add_global_callback(Event::Key(Key::Right), move |_| {
            progress_model.increment_progress();
        });
    }

    {
        let counter_model = counter_model.clone();
        siv.add_global_callback('-', move |_| {
            counter_model.decrement_count();
        });
    }

    {
        let counter_model = counter_model.clone();
        siv.add_global_callback('_', move |_| {
            counter_model.decrement_count();
        });
    }

    {
        let counter_model = counter_model.clone();
        siv.add_global_callback('+', move |_| {
            counter_model.increment_count();
        });
    }
    {
        let counter_model = counter_model.clone();
        siv.add_global_callback('=', move |_| {
            counter_model.increment_count();
        });
    }

    {
        let counter_model = counter_model.clone();
        siv.add_global_callback('o', move |_| {
            counter_model.start_counter();
        });
    }

    {
        let counter_model = counter_model.clone();
        siv.add_global_callback('p', move |_| {
            counter_model.stop_counter();
        });
    }

    {
        let executor_model = executor_model.clone();
        siv.add_global_callback('c', move |_| {
            executor_model.request_calc();
        });
    }
}
