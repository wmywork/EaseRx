use crate::Arc;
use crate::input::input_state::InputState;
use easerx::StateStore;

pub struct InputHandler {
    pub store: Arc<StateStore<InputState>>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            store: Arc::new(StateStore::new(InputState::default())),
        }
    }

    pub fn request_exit(&self) {
        self.store._set_state(|state| InputState {
            exit: true,
            ..state
        });
    }
}
