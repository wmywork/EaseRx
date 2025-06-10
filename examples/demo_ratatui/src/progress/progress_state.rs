use ratatui::prelude::Color;
use easerx::State;

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
