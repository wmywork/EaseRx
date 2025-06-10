use easerx::State;
use ratatui::prelude::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressState {
    pub progress: f64,
    pub color: Color,
}

impl State for ProgressState {}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            progress: 0.5,
            color: Color::Indexed(2),
        }
    }
}
