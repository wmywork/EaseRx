use crate::progress::progress_state::ProgressState;
use easerx::StateStore;
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
        self.store._set_state(|state| state.increment_progress());
    }

    pub fn decrement_progress(&self) {
        self.store._set_state(|state| state.decrement_progress());
    }

    pub fn change_color_up(&self) {
        self.store._set_state(|state| state.increment_color());
    }

    pub fn change_color_down(&self) {
        self.store._set_state(|state| state.decrement_color());
    }

    pub fn reset_progress(&self) {
        self.store._set_state(|state| state.reset_progress());
    }
}
