use crate::progress::progress_state::ProgressState;
use easerx::StateStore;
use std::sync::Arc;

pub struct ProgressModel {
    store: Arc<StateStore<ProgressState>>,
}

impl ProgressModel {
    // Create a new ProgressViewModel with default state
    pub fn new() -> ProgressModel {
        Self {
            store: Arc::new(StateStore::new(ProgressState::default())),
        }
    }

    // Get a clone of the state store
    pub fn store(&self) -> Arc<StateStore<ProgressState>> {
        self.store.clone()
    }

    // Increment progress by 1%
    pub fn increment_progress(&self) {
        self.store._set_state(|state| state.increment_progress());
    }

    // Decrement progress by 1%
    pub fn decrement_progress(&self) {
        self.store._set_state(|state| state.decrement_progress());
    }

    // Cycle to next color
    pub fn change_color_up(&self) {
        self.store._set_state(|state| state.increment_color());
    }

    // Cycle to previous color
    pub fn change_color_down(&self) {
        self.store._set_state(|state| state.decrement_color());
    }

    // Reset progress to 50% and color to default
    pub fn reset_progress(&self) {
        self.store
            ._set_state(|state| state.reset_progress_and_color());
    }
}
