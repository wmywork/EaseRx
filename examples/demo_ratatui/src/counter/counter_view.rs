use crate::counter::counter_state::CounterState;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn counter_view<'a>(counter_state: &CounterState) -> Paragraph<'a> {
    let status = if counter_state.started {
        "[Running]"
    } else {
        ""
    };
    let counter_text = format!("Count: {}", counter_state.count);
    Paragraph::new(counter_text)
        .block(Block::default().title(format!("Counter {}", status)).borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
}
