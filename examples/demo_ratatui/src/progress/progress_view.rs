use crate::progress::progress_state::ProgressState;
use ratatui::prelude::Style;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Gauge};

pub fn progress_view<'a>(progress_state: &ProgressState) -> Gauge<'a> {
    let block = Block::bordered()
        .title("Progress")
        .border_set(border::THICK);

    let progress_bar = Gauge::default()
        .gauge_style(Style::default().fg(progress_state.color))
        .block(block)
        .label(format!("progress：{:.0}%", progress_state.progress * 100.0))
        .ratio(progress_state.progress);

    progress_bar
}
