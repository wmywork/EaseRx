use crate::Arc;
use crate::input::input_state::InputState;
use crossterm::event;
use crossterm::event::Event;
use easerx::StateStore;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::sleep;

pub struct InputHandler {
    pub store: Arc<StateStore<InputState>>,
}

impl InputHandler {
    pub fn new() -> (Self, UnboundedReceiver<Event>) {
        let (tx, rx) = mpsc::unbounded_channel::<crossterm::event::Event>();
        Self::start_input_listener(tx.clone());
        (
            Self {
                store: Arc::new(StateStore::new(InputState::default())),
            },
            rx,
        )
    }

    pub fn start_input_listener(input_tx: UnboundedSender<crossterm::event::Event>) {
        tokio::spawn(async move {
            loop {
                match event::read() {
                    Ok(ev) => {
                        if input_tx.send(ev).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(_) => {
                        // Error reading event, maybe log or handle
                        break;
                    }
                }
            }
        });
    }

    pub fn start_dispatcher<F>(mut input_rx: UnboundedReceiver<Event>, callback: F)
    where
        F: Fn(Event) + std::marker::Send + 'static, // 闭包返回 Future
    {
        tokio::spawn(async move {
            loop {
                match input_rx.try_recv() {
                    Ok(event) => {
                        callback(event);
                    }
                    _ => {}
                }
                sleep(Duration::from_millis(10)).await;
            }
        });
    }

    pub fn request_exit(&self) {
        self.store._set_state(|state| InputState {
            exit: true,
            ..state
        });
    }
}
