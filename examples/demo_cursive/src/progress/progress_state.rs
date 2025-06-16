use easerx::State;
use cursive::theme::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressState {
    pub progress: u8,
    pub color_index: usize,
}

impl State for ProgressState {}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            progress: 50,
            color_index: 2,
        }
    }
}

// Available colors for progress bar
pub const COLORS: [Color; 6] = [
    Color::Dark(cursive::theme::BaseColor::Red),
    Color::Dark(cursive::theme::BaseColor::Green),
    Color::Dark(cursive::theme::BaseColor::Blue),
    Color::Light(cursive::theme::BaseColor::Red),
    Color::Light(cursive::theme::BaseColor::Green),
    Color::Light(cursive::theme::BaseColor::Blue),
];

impl ProgressState {
    // Increment progress by 1%
    pub fn increment_progress(self) -> Self {
        let progress = if self.progress < 99 {
            self.progress + 1
        } else {
            100
        };
        Self { progress, ..self }
    }

    // Decrement progress by 1%
    pub fn decrement_progress(self) -> Self {
        let progress = if self.progress > 1 {
            self.progress - 1
        } else {
            0
        };
        Self { progress, ..self }
    }

    // Reset progress to 50% and color to default
    pub fn reset_progress_and_color(self) -> Self {
        Self {
            progress: 50,
            color_index: 2,
        }
    }

    // Cycle to next color
    pub fn increment_color(self) -> Self {
        let color_index = (self.color_index + 1) % COLORS.len();
        Self { color_index, ..self }
    }

    // Cycle to previous color
    pub fn decrement_color(self) -> Self {
        let color_index = if self.color_index > 0 {
            self.color_index - 1
        } else {
            COLORS.len() - 1
        };
        Self { color_index, ..self }
    }
    
    // Get current color
    pub fn get_color(&self) -> Color {
        COLORS[self.color_index]
    }
} 