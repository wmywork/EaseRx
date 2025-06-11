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

impl ProgressState {
    pub fn increment_progress(self) -> Self {
        let progress = if self.progress < 0.99 {
            self.progress + 0.01
        } else {
            1.0
        };
        Self { progress, ..self }
    }

    pub fn decrement_progress(self) -> Self {
        let progress = if self.progress > 0.01 {
            self.progress - 0.01
        } else {
            0.0
        };
        Self { progress, ..self }
    }

    pub fn reset_progress_and_color(self) -> Self {
        Self {
            progress: 0.5,
            color: Color::Indexed(2),
            ..self
        }
    }

    pub fn increment_color(self) -> Self {
        let color = match self.color {
            Color::Indexed(id) => {
                let id = if id < u8::MAX { id + 1 } else { id };
                Color::Indexed(id)
            }
            _ => Color::Indexed(1),
        };
        Self { color, ..self }
    }

    pub fn decrement_color(self) -> Self {
        let color = match self.color {
            Color::Indexed(id) => {
                let id = if id > u8::MIN + 1 { id - 1 } else { id };
                Color::Indexed(id)
            }
            _ => Color::Indexed(1),
        };
        Self { color, ..self }
    }
}
