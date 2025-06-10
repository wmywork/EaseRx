use crate::progress::progress_state::ProgressState;
use easerx::StateStore;
use ratatui::prelude::Color;
use std::sync::Arc;

pub struct ProgressViewModel {
    store: Arc<StateStore<ProgressState>>,
}

impl ProgressViewModel {
    pub fn new() -> ProgressViewModel {
        Self {
            store: Arc::new(StateStore::new(ProgressState::default())),
        }
    }

    pub fn store(&self) -> Arc<StateStore<ProgressState>> {
        self.store.clone()
    }

    pub fn increment_progress(&self) {
        self.store._set_state(|state| {
            let progress = if state.progress < 0.99 {
                state.progress + 0.01
            } else {
                1.0
            };
            ProgressState { progress, ..state }
        });
    }

    pub fn decrement_progress(&self) {
        self.store._set_state(|state| {
            let progress = if state.progress > 0.01 {
                state.progress - 0.01
            } else {
                0.0
            };
            ProgressState { progress, ..state }
        });
    }

    pub fn change_color_up(&self) {
        self.store._set_state(|state| {
            let color = match state.color {
                Color::Indexed(id) => {
                    let id = if id < u8::MAX { id + 1 } else { id };
                    Color::Indexed(id)
                }
                _ => Color::Indexed(1),
            };
            ProgressState { color, ..state }
        });
    }

    pub fn change_color_down(&self) {
        self.store._set_state(|state| {
            let color = match state.color {
                Color::Indexed(id) => {
                    let id = if id > u8::MIN + 1 { id - 1 } else { id };
                    Color::Indexed(id)
                }
                _ => Color::Indexed(1),
            };
            ProgressState { color, ..state }
        });
    }

    pub fn reset_progress(&self) {
        self.store._set_state(|state| ProgressState {
            progress: 0.5,
            ..state
        });
    }
}
